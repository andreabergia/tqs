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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
}

#[allow(dead_code)] // config, repo, refresh() used in Phase 2
pub struct TuiApp {
    pub config: ResolvedConfig,
    pub repo: TaskRepo,

    // Cached task data
    pub tasks: Vec<Task>,

    // Navigation
    pub active_queue_index: usize,
    pub task_list_state: ListState,

    // Mode
    pub mode: Mode,

    // Status
    pub should_quit: bool,
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
            mode: Mode::Normal,
            should_quit: false,
        };
        app.select_first_task();
        Ok(app)
    }

    #[allow(dead_code)] // used in Phase 2
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
    }

    pub fn select_prev_task(&mut self) {
        let count = self.current_queue_tasks().len();
        if count == 0 {
            return;
        }
        let current = self.task_list_state.selected().unwrap_or(0);
        let prev = if current == 0 { count - 1 } else { current - 1 };
        self.task_list_state.select(Some(prev));
    }

    fn select_first_task(&mut self) {
        if self.current_queue_tasks().is_empty() {
            self.task_list_state.select(None);
        } else {
            self.task_list_state.select(Some(0));
        }
    }
}
