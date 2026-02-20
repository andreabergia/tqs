use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Open,
    Closed,
}

impl TaskStatus {
    pub fn is_open(self) -> bool {
        matches!(self, Self::Open)
    }

    pub fn is_closed(self) -> bool {
        matches!(self, Self::Closed)
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Open => "open",
            Self::Closed => "closed",
        };

        f.write_str(text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub status: TaskStatus,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Task {
    pub fn new(
        id: impl Into<String>,
        created_at: DateTime<Utc>,
        summary: impl Into<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            created_at,
            status: TaskStatus::Open,
            summary: summary.into(),
            description,
        }
    }

    pub fn close(&mut self) -> bool {
        if self.status.is_closed() {
            return false;
        }

        self.status = TaskStatus::Closed;
        true
    }

    pub fn reopen(&mut self) -> bool {
        if self.status.is_open() {
            return false;
        }

        self.status = TaskStatus::Open;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{Task, TaskStatus};
    use chrono::{DateTime, Utc};

    fn created_at() -> DateTime<Utc> {
        "2026-02-20T22:15:00Z"
            .parse()
            .expect("timestamp literal should parse")
    }

    #[test]
    fn task_status_serializes_as_lowercase() {
        let output = serde_yaml::to_string(&TaskStatus::Open).expect("status should serialize");
        assert_eq!(output.trim(), "open");
    }

    #[test]
    fn task_status_deserializes_from_lowercase() {
        let status: TaskStatus = serde_yaml::from_str("closed").expect("status should deserialize");
        assert_eq!(status, TaskStatus::Closed);
    }

    #[test]
    fn close_and_reopen_are_idempotent() {
        let mut task = Task::new("alpha", created_at(), "Write docs", None);
        assert!(task.close());
        assert!(task.status.is_closed());
        assert!(!task.close());

        assert!(task.reopen());
        assert!(task.status.is_open());
        assert!(!task.reopen());
    }

    #[test]
    fn task_serde_roundtrip_preserves_fields() {
        let task = Task {
            id: "cobalt-urial-7f3a".to_string(),
            created_at: created_at(),
            status: TaskStatus::Open,
            summary: "Short task summary".to_string(),
            description: Some("Body".to_string()),
        };

        let yaml = serde_yaml::to_string(&task).expect("task should serialize");
        let parsed: Task = serde_yaml::from_str(&yaml).expect("task should deserialize");
        assert_eq!(parsed, task);
    }
}
