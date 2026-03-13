use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Queue {
    Inbox,
    Now,
    Next,
    Later,
    Done,
}

impl Queue {
    pub const ORDERED: [Queue; 5] = [
        Queue::Inbox,
        Queue::Now,
        Queue::Next,
        Queue::Later,
        Queue::Done,
    ];

    pub fn all() -> &'static [Queue] {
        &Self::ORDERED
    }

    pub fn is_done(self) -> bool {
        matches!(self, Self::Done)
    }
}

impl fmt::Display for Queue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Inbox => "inbox",
            Self::Now => "now",
            Self::Next => "next",
            Self::Later => "later",
            Self::Done => "done",
        };

        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueParseError;

impl fmt::Display for QueueParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid queue")
    }
}

impl std::error::Error for QueueParseError {}

impl FromStr for Queue {
    type Err = QueueParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "inbox" => Ok(Self::Inbox),
            "now" => Ok(Self::Now),
            "next" => Ok(Self::Next),
            "later" => Ok(Self::Later),
            "done" => Ok(Self::Done),
            _ => Err(QueueParseError),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub queue: Queue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_note: Option<String>,
    #[serde(default)]
    pub body: String,
}

impl Task {
    pub fn new(id: impl Into<String>, title: impl Into<String>, now: DateTime<Utc>) -> Self {
        let title = title.into();

        Self {
            id: id.into(),
            title: title.clone(),
            queue: Queue::Inbox,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            completed_at: None,
            daily_note: None,
            body: Self::default_body(&title),
        }
    }

    pub fn default_body(title: &str) -> String {
        format!("# {title}\n")
    }

    pub fn move_to(&mut self, queue: Queue, now: DateTime<Utc>) -> bool {
        if self.queue == queue {
            return false;
        }

        self.queue = queue;
        self.updated_at = now;
        self.completed_at = if queue.is_done() { Some(now) } else { None };
        true
    }

    pub fn normalize(&mut self, now: DateTime<Utc>) {
        self.updated_at = now;
        if self.queue.is_done() {
            if self.completed_at.is_none() {
                self.completed_at = Some(now);
            }
        } else {
            self.completed_at = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Queue, Task};
    use chrono::{DateTime, Utc};

    fn now() -> DateTime<Utc> {
        "2026-03-09T10:34:12Z"
            .parse()
            .expect("timestamp should parse")
    }

    #[test]
    fn queue_roundtrips_in_yaml() {
        let rendered = serde_yaml::to_string(&Queue::Later).expect("queue should serialize");
        let parsed: Queue = serde_yaml::from_str(&rendered).expect("queue should deserialize");
        assert_eq!(parsed, Queue::Later);
    }

    #[test]
    fn queue_rejects_invalid_names() {
        assert!("archive".parse::<Queue>().is_err());
    }

    #[test]
    fn new_task_uses_inbox_defaults() {
        let task = Task::new("task-1", "Ship v2", now());
        assert_eq!(task.queue, Queue::Inbox);
        assert_eq!(task.created_at, now());
        assert_eq!(task.updated_at, now());
        assert!(task.completed_at.is_none());
        assert!(task.body.contains("# Ship v2"));
    }

    #[test]
    fn move_to_done_sets_completed_at() {
        let mut task = Task::new("task-1", "Ship v2", now());
        let changed = task.move_to(
            Queue::Done,
            "2026-03-10T08:00:00Z"
                .parse()
                .expect("timestamp should parse"),
        );

        assert!(changed);
        assert_eq!(task.queue, Queue::Done);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn move_to_same_queue_is_a_noop() {
        let mut task = Task::new("task-1", "Ship v2", now());
        let changed = task.move_to(
            Queue::Inbox,
            "2026-03-10T08:00:00Z"
                .parse()
                .expect("timestamp should parse"),
        );

        assert!(!changed);
        assert_eq!(task.queue, Queue::Inbox);
        assert_eq!(task.updated_at, now());
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn normalize_clears_completed_at_outside_done() {
        let mut task = Task::new("task-1", "Ship v2", now());
        task.completed_at = Some(now());
        task.normalize(
            "2026-03-10T08:00:00Z"
                .parse()
                .expect("timestamp should parse"),
        );

        assert!(task.completed_at.is_none());
    }

    #[test]
    fn normalize_sets_completed_at_for_done_tasks_without_it() {
        let mut task = Task::new("task-1", "Ship v2", now());
        let normalized_at = "2026-03-10T08:00:00Z"
            .parse()
            .expect("timestamp should parse");
        task.queue = Queue::Done;

        task.normalize(normalized_at);

        assert_eq!(task.updated_at, normalized_at);
        assert_eq!(task.completed_at, Some(normalized_at));
    }
}
