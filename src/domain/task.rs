use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub status: TaskStatus,
    pub summary: String,
    pub description: Option<String>,
}
