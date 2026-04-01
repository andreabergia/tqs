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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    AddForm,
    ConfirmDelete { task_id: String, from_triage: bool },
    MoveTarget { from_triage: bool },
    Search,
    Triage,
}

pub use crate::app::operations::TriageSummary;

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

    // Add form state
    pub add_title: String,
    pub add_queue: Queue,

    // Search state
    pub search_query: String,
    pub search_results: Vec<(String, Queue)>, // (task_id, queue) pairs
    pub search_list_state: ListState,

    // Triage state
    pub triage_task_ids: Vec<String>,
    pub triage_index: usize,
    pub triage_summary: TriageSummary,

    // Transient status message
    pub status_message: Option<(String, Instant)>,
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
            add_title: String::new(),
            add_queue: Queue::Inbox,
            search_query: String::new(),
            search_results: Vec::new(),
            search_list_state: ListState::default(),
            triage_task_ids: Vec::new(),
            triage_index: 0,
            triage_summary: TriageSummary::default(),
            status_message: None,
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

    pub fn queue_count(&self, queue: Queue) -> usize {
        self.tasks.iter().filter(|t| t.queue == queue).count()
    }

    pub fn total_count(&self) -> usize {
        self.tasks.len()
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
        self.select_first_task();
    }

    pub fn prev_queue(&mut self) {
        self.active_sidebar_index = next_selectable(self.active_sidebar_index, -1);
        self.select_first_task();
    }

    pub fn select_queue_by_index(&mut self, index: usize) {
        // Map 1-5 to the first 5 selectable entries
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
        self.search_results = self
            .tasks
            .iter()
            .filter(|t| matches_query(t, &self.search_query))
            .map(|t| (t.id.clone(), t.queue))
            .collect();
        if self.search_results.is_empty() {
            self.search_list_state.select(None);
        } else {
            self.search_list_state.select(Some(0));
        }
    }

    pub fn select_search_result(&mut self) {
        let Some(idx) = self.search_list_state.selected() else {
            return;
        };
        let Some((task_id, queue)) = self.search_results.get(idx).cloned() else {
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
        let task_id = self.triage_task_ids.get(self.triage_index)?;
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
        self.triage_task_ids = inbox_ids;
        self.triage_index = 0;
        self.triage_summary = TriageSummary::default();
        self.mode = Mode::Triage;
    }

    pub fn advance_triage_or_finish(&mut self) {
        self.triage_index += 1;
        if self.triage_index >= self.triage_task_ids.len() {
            let summary = self.triage_summary.to_string();
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
