use std::fmt;
use std::time::Instant;

use ratatui::widgets::ListState;

use crate::app::app_error::AppError;
use crate::domain::task::{Queue, Task};
use crate::storage::config::ResolvedConfig;
use crate::storage::repo::TaskRepo;

/// What the sidebar can show: a queue, a separator line, or "all".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarEntry {
    Queue(Queue),
    Separator,
    All,
}

/// The sidebar layout with visual separators between groups.
const SIDEBAR_ENTRIES: &[SidebarEntry] = &[
    SidebarEntry::Queue(Queue::Now),
    SidebarEntry::Queue(Queue::Next),
    SidebarEntry::Queue(Queue::Later),
    SidebarEntry::Separator,
    SidebarEntry::Queue(Queue::Inbox),
    SidebarEntry::Separator,
    SidebarEntry::Queue(Queue::Done),
    SidebarEntry::All,
];

/// Which tasks to show in the task list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueFilter {
    Single(Queue),
    All,
}

impl fmt::Display for QueueFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(q) => write!(f, "{q}"),
            Self::All => write!(f, "all"),
        }
    }
}

pub struct QueueCounts {
    counts: [usize; 5],
    pub total: usize,
}

impl QueueCounts {
    pub fn get(&self, queue: Queue) -> usize {
        let idx = match queue {
            Queue::Inbox => 0,
            Queue::Now => 1,
            Queue::Next => 2,
            Queue::Later => 3,
            Queue::Done => 4,
        };
        self.counts[idx]
    }
}

/// How long status messages stay visible.
const STATUS_MESSAGE_TTL_SECS: u64 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Sidebar,
    TaskList,
    Detail,
}

impl FocusedPanel {
    pub fn left(self) -> Self {
        match self {
            Self::Detail => Self::TaskList,
            Self::TaskList => Self::Sidebar,
            Self::Sidebar => Self::Sidebar,
        }
    }

    pub fn right(self) -> Self {
        match self {
            Self::Sidebar => Self::TaskList,
            Self::TaskList => Self::Detail,
            Self::Detail => Self::Detail,
        }
    }
}

pub use crate::app::operations::TriageSummary;

pub enum Mode {
    Normal,
    AddForm {
        title: String,
        queue: Queue,
    },
    ConfirmDelete {
        task_id: String,
        from_triage: bool,
    },
    MoveTarget {
        from_triage: bool,
    },
    Search {
        query: String,
        results: Vec<(String, Queue)>,
        list_state: ListState,
    },
    Triage,
}

/// Triage state lives outside Mode because it must survive transitions
/// to ConfirmDelete and MoveTarget sub-modes.
#[derive(Default)]
pub struct TriageState {
    pub task_ids: Vec<String>,
    pub index: usize,
    pub summary: TriageSummary,
}

pub struct TuiApp {
    pub config: ResolvedConfig,
    pub repo: TaskRepo,

    // Cached task data
    pub tasks: Vec<Task>,

    // Navigation
    pub active_sidebar_index: usize,
    pub task_list_state: ListState,

    // Panel focus
    pub focused_panel: FocusedPanel,
    pub detail_scroll: u16,

    // Mode
    pub mode: Mode,

    // Triage state (persists across ConfirmDelete/MoveTarget sub-modes)
    pub triage: TriageState,

    // Transient status message
    pub status_message: Option<(String, Instant)>,

    // Redraw flag — set when state changes, cleared after draw
    pub needs_redraw: bool,
}

impl TuiApp {
    pub fn new(config: ResolvedConfig, repo: TaskRepo) -> Result<Self, AppError> {
        let tasks = repo.list()?;
        let mut app = Self {
            config,
            repo,
            tasks,
            active_sidebar_index: 0,
            task_list_state: ListState::default(),
            focused_panel: FocusedPanel::TaskList,
            detail_scroll: 0,
            mode: Mode::Normal,
            triage: TriageState::default(),
            status_message: None,
            needs_redraw: true,
        };
        app.select_first_task();
        Ok(app)
    }

    pub fn refresh(&mut self) -> Result<(), AppError> {
        self.tasks = self.repo.list()?;
        let count = self.current_queue_tasks().len();
        if count == 0 {
            self.task_list_state.select(None);
        } else if let Some(i) = self.task_list_state.selected()
            && i >= count
        {
            self.task_list_state.select(Some(count - 1));
        }
        Ok(())
    }

    pub fn sidebar_entries(&self) -> &[SidebarEntry] {
        SIDEBAR_ENTRIES
    }

    pub fn active_filter(&self) -> QueueFilter {
        match SIDEBAR_ENTRIES[self.active_sidebar_index] {
            SidebarEntry::Queue(q) => QueueFilter::Single(q),
            SidebarEntry::All => QueueFilter::All,
            SidebarEntry::Separator => unreachable!("separator cannot be active sidebar entry"),
        }
    }

    pub fn queue_counts(&self) -> QueueCounts {
        let mut counts = [0usize; 5];
        for task in &self.tasks {
            let idx = match task.queue {
                Queue::Inbox => 0,
                Queue::Now => 1,
                Queue::Next => 2,
                Queue::Later => 3,
                Queue::Done => 4,
            };
            counts[idx] += 1;
        }
        QueueCounts {
            counts,
            total: self.tasks.len(),
        }
    }

    pub fn current_queue_tasks(&self) -> Vec<&Task> {
        match self.active_filter() {
            QueueFilter::Single(queue) => self.tasks.iter().filter(|t| t.queue == queue).collect(),
            QueueFilter::All => self.tasks.iter().collect(),
        }
    }

    pub fn selected_task(&self) -> Option<&Task> {
        let tasks = self.current_queue_tasks();
        self.task_list_state
            .selected()
            .and_then(|i| tasks.get(i).copied())
    }

    pub fn next_queue(&mut self) {
        self.active_sidebar_index = next_selectable(self.active_sidebar_index, 1);
        debug_assert!(
            !matches!(
                SIDEBAR_ENTRIES[self.active_sidebar_index],
                SidebarEntry::Separator
            ),
            "next_queue landed on a separator"
        );
        self.select_first_task();
    }

    pub fn prev_queue(&mut self) {
        self.active_sidebar_index = next_selectable(self.active_sidebar_index, -1);
        debug_assert!(
            !matches!(
                SIDEBAR_ENTRIES[self.active_sidebar_index],
                SidebarEntry::Separator
            ),
            "prev_queue landed on a separator"
        );
        self.select_first_task();
    }

    pub fn select_queue_by_index(&mut self, index: usize) {
        let selectable: Vec<usize> = SIDEBAR_ENTRIES
            .iter()
            .enumerate()
            .filter(|(_, e)| matches!(e, SidebarEntry::Queue(_) | SidebarEntry::All))
            .map(|(i, _)| i)
            .collect();
        if let Some(&sidebar_idx) = selectable.get(index) {
            self.active_sidebar_index = sidebar_idx;
            self.select_first_task();
        }
    }

    pub fn jump_to_queue(&mut self, queue: Queue) {
        if let Some(idx) = SIDEBAR_ENTRIES
            .iter()
            .position(|e| *e == SidebarEntry::Queue(queue))
        {
            self.active_sidebar_index = idx;
            self.select_first_task();
        }
    }

    pub fn select_next_task(&mut self) {
        let count = self.current_queue_tasks().len();
        if count == 0 {
            return;
        }
        let current = self.task_list_state.selected().unwrap_or(0);
        let next = if current + 1 >= count { 0 } else { current + 1 };
        self.task_list_state.select(Some(next));
        self.detail_scroll = 0;
    }

    pub fn select_prev_task(&mut self) {
        let count = self.current_queue_tasks().len();
        if count == 0 {
            return;
        }
        let current = self.task_list_state.selected().unwrap_or(0);
        let prev = if current == 0 { count - 1 } else { current - 1 };
        self.task_list_state.select(Some(prev));
        self.detail_scroll = 0;
    }

    pub fn update_search_results(&mut self) {
        use crate::domain::filter::matches_query;
        let Mode::Search {
            query,
            results,
            list_state,
        } = &mut self.mode
        else {
            return;
        };
        *results = self
            .tasks
            .iter()
            .filter(|t| matches_query(t, query))
            .map(|t| (t.id.clone(), t.queue))
            .collect();
        if results.is_empty() {
            list_state.select(None);
        } else {
            list_state.select(Some(0));
        }
    }

    pub fn select_search_result(&mut self) {
        let Mode::Search {
            results,
            list_state,
            ..
        } = &self.mode
        else {
            return;
        };
        let Some(idx) = list_state.selected() else {
            return;
        };
        let Some((task_id, queue)) = results.get(idx).cloned() else {
            return;
        };
        self.jump_to_queue(queue);
        let task_index = self
            .tasks
            .iter()
            .filter(|t| t.queue == queue)
            .position(|t| t.id == task_id)
            .unwrap_or(0);
        self.task_list_state.select(Some(task_index));
        self.detail_scroll = 0;
        self.mode = Mode::Normal;
    }

    pub fn current_triage_task(&self) -> Option<&Task> {
        let task_id = self.triage.task_ids.get(self.triage.index)?;
        self.tasks.iter().find(|t| t.id == *task_id)
    }

    pub fn enter_triage(&mut self) {
        let inbox_ids: Vec<String> = self
            .tasks
            .iter()
            .filter(|t| t.queue == Queue::Inbox)
            .map(|t| t.id.clone())
            .collect();
        if inbox_ids.is_empty() {
            self.set_status("Inbox is empty — nothing to triage");
            return;
        }
        self.triage = TriageState {
            task_ids: inbox_ids,
            index: 0,
            summary: TriageSummary::default(),
        };
        self.mode = Mode::Triage;
    }

    pub fn advance_triage_or_finish(&mut self) {
        self.triage.index += 1;
        if self.triage.index >= self.triage.task_ids.len() {
            let summary = self.triage.summary.to_string();
            self.mode = Mode::Normal;
            self.set_status(format!("Triage: {summary}"));
        }
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some((message.into(), Instant::now()));
    }

    pub fn active_status_message(&self) -> Option<&str> {
        self.status_message.as_ref().and_then(|(msg, when)| {
            if when.elapsed().as_secs() < STATUS_MESSAGE_TTL_SECS {
                Some(msg.as_str())
            } else {
                None
            }
        })
    }

    fn select_first_task(&mut self) {
        if self.current_queue_tasks().is_empty() {
            self.task_list_state.select(None);
        } else {
            self.task_list_state.select(Some(0));
        }
        self.detail_scroll = 0;
    }
}

/// Find the next selectable sidebar index (skipping separators), wrapping around.
fn next_selectable(current: usize, direction: i32) -> usize {
    let len = SIDEBAR_ENTRIES.len();
    let mut idx = current;
    loop {
        idx = ((idx as i32 + direction).rem_euclid(len as i32)) as usize;
        if !matches!(SIDEBAR_ENTRIES[idx], SidebarEntry::Separator) {
            return idx;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::config::{QueueDirs, ResolvedConfig};
    use crate::storage::repo::TaskRepo;
    use chrono::Utc;
    use ratatui::widgets::ListState;
    use tempfile::TempDir;

    fn make_app(temp: &TempDir) -> TuiApp {
        let root = temp.path().to_path_buf();
        let config = ResolvedConfig {
            obsidian_vault_dir: None,
            tasks_root: root.clone(),
            state_dir: root.join(".tqs"),
            daily_notes_dir: None,
            queue_dirs: QueueDirs::default(),
        };
        let repo = TaskRepo::new(root, QueueDirs::default());
        TuiApp::new(config, repo).unwrap()
    }

    fn make_app_with_tasks(temp: &TempDir, tasks: &[(&str, Queue)]) -> TuiApp {
        let root = temp.path().to_path_buf();
        let config = ResolvedConfig {
            obsidian_vault_dir: None,
            tasks_root: root.clone(),
            state_dir: root.join(".tqs"),
            daily_notes_dir: None,
            queue_dirs: QueueDirs::default(),
        };
        let repo = TaskRepo::new(root.clone(), QueueDirs::default());
        for (id, queue) in tasks {
            let mut task = Task::new(id.to_string(), &format!("Task {id}"), Utc::now());
            task.queue = *queue;
            repo.create(&task).unwrap();
        }
        TuiApp::new(config, repo).unwrap()
    }

    // --- FocusedPanel ---

    #[test]
    fn focused_panel_left_clamps_at_sidebar() {
        assert_eq!(FocusedPanel::Sidebar.left(), FocusedPanel::Sidebar);
        assert_eq!(FocusedPanel::TaskList.left(), FocusedPanel::Sidebar);
        assert_eq!(FocusedPanel::Detail.left(), FocusedPanel::TaskList);
    }

    #[test]
    fn focused_panel_right_clamps_at_detail() {
        assert_eq!(FocusedPanel::Sidebar.right(), FocusedPanel::TaskList);
        assert_eq!(FocusedPanel::TaskList.right(), FocusedPanel::Detail);
        assert_eq!(FocusedPanel::Detail.right(), FocusedPanel::Detail);
    }

    // --- next_selectable ---

    #[test]
    fn next_selectable_skips_separators() {
        // SIDEBAR_ENTRIES: Now(0), Next(1), Later(2), Sep(3), Inbox(4), Sep(5), Done(6), All(7)
        // From Later(2), next should skip Sep(3) and land on Inbox(4)
        assert_eq!(next_selectable(2, 1), 4);
        // From Inbox(4), prev should skip Sep(3) and land on Later(2)
        assert_eq!(next_selectable(4, -1), 2);
    }

    #[test]
    fn next_selectable_wraps_around() {
        // From All(7), next should wrap to Now(0)
        assert_eq!(next_selectable(7, 1), 0);
        // From Now(0), prev should wrap to All(7)
        assert_eq!(next_selectable(0, -1), 7);
    }

    // --- TuiApp::new ---

    #[test]
    fn new_app_starts_on_first_queue_with_task_list_focused() {
        let temp = TempDir::new().unwrap();
        let app = make_app(&temp);
        assert_eq!(app.active_sidebar_index, 0);
        assert_eq!(app.focused_panel, FocusedPanel::TaskList);
        assert!(matches!(app.mode, Mode::Normal));
        assert!(app.needs_redraw);
    }

    #[test]
    fn new_app_selects_first_task_when_tasks_exist() {
        let temp = TempDir::new().unwrap();
        let app = make_app_with_tasks(&temp, &[("a1", Queue::Now), ("a2", Queue::Now)]);
        assert_eq!(app.task_list_state.selected(), Some(0));
    }

    #[test]
    fn new_app_no_selection_when_queue_empty() {
        let temp = TempDir::new().unwrap();
        let app = make_app(&temp);
        // Default sidebar is Now, which is empty
        assert_eq!(app.task_list_state.selected(), None);
    }

    // --- active_filter ---

    #[test]
    fn active_filter_returns_queue_for_queue_entries() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app(&temp);
        app.active_sidebar_index = 0; // Now
        assert_eq!(app.active_filter(), QueueFilter::Single(Queue::Now));
    }

    #[test]
    fn active_filter_returns_all_for_all_entry() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app(&temp);
        app.active_sidebar_index = 7; // All
        assert_eq!(app.active_filter(), QueueFilter::All);
    }

    // --- queue_counts ---

    #[test]
    fn queue_counts_single_pass() {
        let temp = TempDir::new().unwrap();
        let app = make_app_with_tasks(
            &temp,
            &[("a1", Queue::Now), ("a2", Queue::Now), ("a3", Queue::Inbox)],
        );
        let counts = app.queue_counts();
        assert_eq!(counts.get(Queue::Now), 2);
        assert_eq!(counts.get(Queue::Inbox), 1);
        assert_eq!(counts.get(Queue::Next), 0);
        assert_eq!(counts.total, 3);
    }

    // --- current_queue_tasks ---

    #[test]
    fn current_queue_tasks_filters_by_active_queue() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now), ("a2", Queue::Inbox)]);
        app.active_sidebar_index = 0; // Now
        assert_eq!(app.current_queue_tasks().len(), 1);
        assert_eq!(app.current_queue_tasks()[0].id, "a1");
    }

    #[test]
    fn current_queue_tasks_returns_all_when_all_selected() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now), ("a2", Queue::Inbox)]);
        app.active_sidebar_index = 7; // All
        assert_eq!(app.current_queue_tasks().len(), 2);
    }

    // --- selected_task ---

    #[test]
    fn selected_task_returns_none_when_no_selection() {
        let temp = TempDir::new().unwrap();
        let app = make_app(&temp);
        assert!(app.selected_task().is_none());
    }

    #[test]
    fn selected_task_returns_task_at_index() {
        let temp = TempDir::new().unwrap();
        let app = make_app_with_tasks(&temp, &[("a1", Queue::Now)]);
        assert_eq!(app.selected_task().unwrap().id, "a1");
    }

    // --- queue navigation ---

    #[test]
    fn next_queue_advances_and_wraps() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app(&temp);
        assert_eq!(app.active_sidebar_index, 0); // Now
        app.next_queue();
        assert_eq!(app.active_sidebar_index, 1); // Next
        app.next_queue();
        assert_eq!(app.active_sidebar_index, 2); // Later
        app.next_queue();
        // Should skip separator(3) and land on Inbox(4)
        assert_eq!(app.active_sidebar_index, 4);
    }

    #[test]
    fn prev_queue_goes_back_and_wraps() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app(&temp);
        app.active_sidebar_index = 0; // Now
        app.prev_queue();
        // Should wrap to All(7)
        assert_eq!(app.active_sidebar_index, 7);
    }

    #[test]
    fn select_queue_by_index_maps_to_selectable_entries() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app(&temp);
        app.select_queue_by_index(0); // First selectable = Now(0)
        assert_eq!(app.active_sidebar_index, 0);
        app.select_queue_by_index(3); // Fourth selectable = Inbox(4)
        assert_eq!(app.active_sidebar_index, 4);
    }

    #[test]
    fn jump_to_queue_sets_correct_sidebar_index() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app(&temp);
        app.jump_to_queue(Queue::Inbox);
        assert_eq!(app.active_sidebar_index, 4);
        app.jump_to_queue(Queue::Done);
        assert_eq!(app.active_sidebar_index, 6);
    }

    // --- task navigation ---

    #[test]
    fn select_next_task_wraps_around() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now), ("a2", Queue::Now)]);
        assert_eq!(app.task_list_state.selected(), Some(0));
        app.select_next_task();
        assert_eq!(app.task_list_state.selected(), Some(1));
        app.select_next_task();
        assert_eq!(app.task_list_state.selected(), Some(0)); // wrapped
    }

    #[test]
    fn select_prev_task_wraps_around() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now), ("a2", Queue::Now)]);
        assert_eq!(app.task_list_state.selected(), Some(0));
        app.select_prev_task();
        assert_eq!(app.task_list_state.selected(), Some(1)); // wrapped
    }

    #[test]
    fn select_next_task_noop_when_empty() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app(&temp);
        app.select_next_task(); // should not panic
        assert_eq!(app.task_list_state.selected(), None);
    }

    #[test]
    fn task_navigation_resets_detail_scroll() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now), ("a2", Queue::Now)]);
        app.detail_scroll = 5;
        app.select_next_task();
        assert_eq!(app.detail_scroll, 0);
    }

    // --- refresh ---

    #[test]
    fn refresh_clamps_selection_when_tasks_removed() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now), ("a2", Queue::Now)]);
        app.task_list_state.select(Some(1)); // select second task
        app.repo.delete("a2").unwrap();
        app.refresh().unwrap();
        // Selection should clamp to 0 (only task remaining)
        assert_eq!(app.task_list_state.selected(), Some(0));
    }

    #[test]
    fn refresh_clears_selection_when_queue_becomes_empty() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now)]);
        app.repo.delete("a1").unwrap();
        app.refresh().unwrap();
        assert_eq!(app.task_list_state.selected(), None);
    }

    // --- search ---

    #[test]
    fn update_search_results_filters_tasks() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now), ("b1", Queue::Inbox)]);
        app.mode = Mode::Search {
            query: "Task a1".to_string(),
            results: Vec::new(),
            list_state: ListState::default(),
        };
        app.update_search_results();
        if let Mode::Search {
            results,
            list_state,
            ..
        } = &app.mode
        {
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].0, "a1");
            assert_eq!(list_state.selected(), Some(0));
        } else {
            panic!("expected Search mode");
        }
    }

    #[test]
    fn update_search_results_clears_selection_when_no_matches() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now)]);
        app.mode = Mode::Search {
            query: "nonexistent".to_string(),
            results: Vec::new(),
            list_state: ListState::default(),
        };
        app.update_search_results();
        if let Mode::Search {
            results,
            list_state,
            ..
        } = &app.mode
        {
            assert!(results.is_empty());
            assert_eq!(list_state.selected(), None);
        } else {
            panic!("expected Search mode");
        }
    }

    #[test]
    fn update_search_results_noop_outside_search_mode() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app(&temp);
        // Should not panic when not in Search mode
        app.update_search_results();
    }

    #[test]
    fn select_search_result_jumps_to_task() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Inbox), ("b1", Queue::Now)]);
        app.mode = Mode::Search {
            query: String::new(),
            results: vec![("a1".to_string(), Queue::Inbox)],
            list_state: ListState::default().with_selected(Some(0)),
        };
        app.select_search_result();
        assert!(matches!(app.mode, Mode::Normal));
        assert_eq!(app.active_sidebar_index, 4); // Inbox
    }

    // --- triage ---

    #[test]
    fn enter_triage_sets_mode_with_inbox_tasks() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(
            &temp,
            &[
                ("a1", Queue::Inbox),
                ("a2", Queue::Inbox),
                ("b1", Queue::Now),
            ],
        );
        app.enter_triage();
        assert!(matches!(app.mode, Mode::Triage));
        assert_eq!(app.triage.task_ids.len(), 2);
        assert_eq!(app.triage.index, 0);
    }

    #[test]
    fn enter_triage_sets_status_when_inbox_empty() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app(&temp);
        app.enter_triage();
        assert!(matches!(app.mode, Mode::Normal));
        assert!(app.active_status_message().unwrap().contains("empty"));
    }

    #[test]
    fn current_triage_task_returns_first_inbox_task() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Inbox), ("a2", Queue::Inbox)]);
        app.enter_triage();
        let task = app.current_triage_task().unwrap();
        // First triage task should be one of the inbox tasks
        assert!(task.queue == Queue::Inbox);
        assert_eq!(task.id, app.triage.task_ids[0]);
    }

    #[test]
    fn advance_triage_moves_to_next_task() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Inbox), ("a2", Queue::Inbox)]);
        app.enter_triage();
        app.advance_triage_or_finish();
        assert!(matches!(app.mode, Mode::Triage));
        assert_eq!(app.triage.index, 1);
    }

    #[test]
    fn advance_triage_finishes_and_shows_summary() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Inbox)]);
        app.enter_triage();
        app.advance_triage_or_finish();
        assert!(matches!(app.mode, Mode::Normal));
        assert!(app.active_status_message().unwrap().contains("Triage"));
    }

    // --- status messages ---

    #[test]
    fn set_status_and_read_back() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app(&temp);
        app.set_status("hello");
        assert_eq!(app.active_status_message(), Some("hello"));
    }

    #[test]
    fn status_message_none_when_not_set() {
        let temp = TempDir::new().unwrap();
        let app = make_app(&temp);
        assert_eq!(app.active_status_message(), None);
    }

    // --- QueueFilter Display ---

    #[test]
    fn queue_filter_display() {
        assert_eq!(QueueFilter::Single(Queue::Now).to_string(), "now");
        assert_eq!(QueueFilter::All.to_string(), "all");
    }
}
