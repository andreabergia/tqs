use std::time::Instant;

use ratatui::widgets::ListState;

use crate::app::app_error::AppError;
use crate::domain::task::{Queue, Task};
use crate::storage::config::ResolvedConfig;
use crate::storage::repo::TaskRepo;

/// Display order for queues in the sidebar.
const SIDEBAR_QUEUES: [Queue; 5] = [
    Queue::Now,
    Queue::Next,
    Queue::Later,
    Queue::Inbox,
    Queue::Done,
];

/// How long status messages stay visible.
const STATUS_MESSAGE_TTL_SECS: u64 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    AddForm,
    ConfirmDelete { task_id: String },
    MoveTarget,
    Search,
    Triage,
}

#[derive(Debug, Default)]
pub struct TriageSummary {
    pub moved_now: u32,
    pub moved_next: u32,
    pub moved_later: u32,
    pub moved_done: u32,
    pub deleted: u32,
    pub skipped: u32,
}

impl TriageSummary {
    pub fn record_move(&mut self, queue: Queue) {
        match queue {
            Queue::Now => self.moved_now += 1,
            Queue::Next => self.moved_next += 1,
            Queue::Later => self.moved_later += 1,
            Queue::Done => self.moved_done += 1,
            Queue::Inbox => {}
        }
    }

    pub fn format(&self) -> String {
        let mut parts = Vec::new();
        if self.moved_now > 0 {
            parts.push(format!("{} to now", self.moved_now));
        }
        if self.moved_next > 0 {
            parts.push(format!("{} to next", self.moved_next));
        }
        if self.moved_later > 0 {
            parts.push(format!("{} to later", self.moved_later));
        }
        if self.moved_done > 0 {
            parts.push(format!("{} done", self.moved_done));
        }
        if self.deleted > 0 {
            parts.push(format!("{} deleted", self.deleted));
        }
        if self.skipped > 0 {
            parts.push(format!("{} skipped", self.skipped));
        }
        if parts.is_empty() {
            "No changes".to_string()
        } else {
            parts.join(", ")
        }
    }
}

pub struct TuiApp {
    pub config: ResolvedConfig,
    pub repo: TaskRepo,

    // Cached task data
    pub tasks: Vec<Task>,

    // Navigation
    pub active_queue_index: usize,
    pub task_list_state: ListState,

    // Detail pane
    pub detail_visible: bool,
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
            active_queue_index: 0,
            task_list_state: ListState::default(),
            detail_visible: false,
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
        // Clamp selection to current queue's task count
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

    pub fn sidebar_queues(&self) -> &[Queue] {
        &SIDEBAR_QUEUES
    }

    pub fn active_queue(&self) -> Queue {
        SIDEBAR_QUEUES[self.active_queue_index]
    }

    pub fn queue_count(&self, queue: Queue) -> usize {
        self.tasks.iter().filter(|t| t.queue == queue).count()
    }

    pub fn current_queue_tasks(&self) -> Vec<&Task> {
        let queue = self.active_queue();
        self.tasks.iter().filter(|t| t.queue == queue).collect()
    }

    pub fn selected_task(&self) -> Option<&Task> {
        let tasks = self.current_queue_tasks();
        self.task_list_state
            .selected()
            .and_then(|i| tasks.get(i).copied())
    }

    pub fn next_queue(&mut self) {
        self.active_queue_index = (self.active_queue_index + 1) % SIDEBAR_QUEUES.len();
        self.select_first_task();
    }

    pub fn prev_queue(&mut self) {
        self.active_queue_index = if self.active_queue_index == 0 {
            SIDEBAR_QUEUES.len() - 1
        } else {
            self.active_queue_index - 1
        };
        self.select_first_task();
    }

    pub fn select_queue(&mut self, index: usize) {
        if index < SIDEBAR_QUEUES.len() {
            self.active_queue_index = index;
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
        // Jump to the queue and select the task
        if let Some(qi) = self.sidebar_queues().iter().position(|q| *q == queue) {
            self.active_queue_index = qi;
            let task_index = self
                .tasks
                .iter()
                .filter(|t| t.queue == queue)
                .position(|t| t.id == task_id)
                .unwrap_or(0);
            self.task_list_state.select(Some(task_index));
            self.detail_scroll = 0;
        }
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
            let summary = self.triage_summary.format();
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
