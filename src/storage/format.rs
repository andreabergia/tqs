use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::task::{Queue, Task};

const FRONTMATTER_DELIMITER: &str = "---";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTaskFile {
    pub frontmatter: String,
    pub body: String,
}

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("missing frontmatter start delimiter")]
    MissingFrontmatter,
    #[error("missing frontmatter end delimiter")]
    MissingFrontmatterEnd,
    #[error("invalid frontmatter yaml: {0}")]
    InvalidFrontmatter(#[from] serde_yaml::Error),
    #[error("task id does not match filename")]
    IdMismatch,
    #[error("completed_at can only be set for done tasks")]
    CompletedAtWithoutDoneQueue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskFrontmatter {
    id: String,
    title: String,
    queue: Queue,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[serde(default)]
    completed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    daily_note: Option<String>,
}

impl From<TaskFrontmatter> for Task {
    fn from(frontmatter: TaskFrontmatter) -> Self {
        Self {
            id: frontmatter.id,
            title: frontmatter.title,
            queue: frontmatter.queue,
            created_at: frontmatter.created_at,
            updated_at: frontmatter.updated_at,
            completed_at: frontmatter.completed_at,
            daily_note: frontmatter.daily_note,
            body: String::new(),
        }
    }
}

impl From<&Task> for TaskFrontmatter {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id.clone(),
            title: task.title.clone(),
            queue: task.queue,
            created_at: task.created_at,
            updated_at: task.updated_at,
            completed_at: task.completed_at,
            daily_note: task.daily_note.clone(),
        }
    }
}

pub fn parse_task_file(input: &str) -> Result<ParsedTaskFile, FormatError> {
    let mut lines = input.lines();
    let Some(first_line) = lines.next() else {
        return Err(FormatError::MissingFrontmatter);
    };

    if first_line.trim() != FRONTMATTER_DELIMITER {
        return Err(FormatError::MissingFrontmatter);
    }

    let mut frontmatter_lines = Vec::new();
    for line in lines.by_ref() {
        if line.trim() == FRONTMATTER_DELIMITER {
            let body = lines.collect::<Vec<_>>().join("\n");
            return Ok(ParsedTaskFile {
                frontmatter: frontmatter_lines.join("\n"),
                body,
            });
        }
        frontmatter_lines.push(line);
    }

    Err(FormatError::MissingFrontmatterEnd)
}

pub fn parse_task_markdown(input: &str) -> Result<Task, FormatError> {
    let parsed = parse_task_file(input)?;
    let frontmatter: TaskFrontmatter = serde_yaml::from_str(&parsed.frontmatter)?;
    let mut task = Task::from(frontmatter);
    task.body = parsed.body;
    validate_task(&task)?;
    Ok(task)
}

pub fn render_task_markdown(task: &Task) -> Result<String, FormatError> {
    validate_task(task)?;

    let frontmatter = TaskFrontmatter::from(task);
    let yaml = serde_yaml::to_string(&frontmatter)?;
    let mut output = String::new();
    output.push_str(FRONTMATTER_DELIMITER);
    output.push('\n');
    output.push_str(&yaml);
    output.push_str(FRONTMATTER_DELIMITER);

    if !task.body.is_empty() {
        output.push('\n');
        output.push_str(&task.body);
    }

    Ok(output)
}

fn validate_task(task: &Task) -> Result<(), FormatError> {
    if task.completed_at.is_some() && !task.queue.is_done() {
        return Err(FormatError::CompletedAtWithoutDoneQueue);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FormatError, parse_task_markdown, render_task_markdown};
    use crate::domain::task::{Queue, Task};

    fn task() -> Task {
        let mut task = Task::new(
            "20260309-103412-reply-aws-billing",
            "Reply to AWS billing alert",
            "2026-03-09T10:34:12Z"
                .parse()
                .expect("timestamp should parse"),
        );
        task.queue = Queue::Now;
        task.updated_at = "2026-03-09T11:20:07Z"
            .parse()
            .expect("timestamp should parse");
        task.body = "# Reply to AWS billing alert\n\n## Notes\n\nCheck Cost Explorer.".to_string();
        task
    }

    #[test]
    fn markdown_roundtrip_preserves_schema_and_body() {
        let task = task();
        let markdown = render_task_markdown(&task).expect("task should render");
        let parsed = parse_task_markdown(&markdown).expect("markdown should parse");
        assert_eq!(parsed, task);
    }

    #[test]
    fn render_rejects_completed_at_for_non_done_task() {
        let mut task = task();
        task.completed_at = Some(task.updated_at);
        let err = render_task_markdown(&task).expect_err("task should fail validation");
        assert!(matches!(err, FormatError::CompletedAtWithoutDoneQueue));
    }

    #[test]
    fn parse_keeps_empty_body_empty() {
        let markdown = "---\nid: task-1\ntitle: Ship v2\nqueue: inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ncompleted_at: null\ndaily_note: null\n---\n";
        let parsed = parse_task_markdown(markdown).expect("markdown should parse");
        assert!(parsed.body.is_empty());
    }

    #[test]
    fn parse_rejects_missing_frontmatter_start_delimiter() {
        let markdown = "id: task-1\ntitle: Ship v2\nqueue: inbox\n";
        let err = parse_task_markdown(markdown).expect_err("markdown should fail to parse");
        assert!(matches!(err, FormatError::MissingFrontmatter));
    }

    #[test]
    fn parse_rejects_missing_frontmatter_end_delimiter() {
        let markdown = "---\nid: task-1\ntitle: Ship v2\nqueue: inbox\n";
        let err = parse_task_markdown(markdown).expect_err("markdown should fail to parse");
        assert!(matches!(err, FormatError::MissingFrontmatterEnd));
    }

    #[test]
    fn parse_rejects_invalid_yaml_frontmatter() {
        let markdown = "---\nid: task-1\ntitle: Ship v2\nqueue: [inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\n---\n";
        let err = parse_task_markdown(markdown).expect_err("markdown should fail to parse");
        assert!(matches!(err, FormatError::InvalidFrontmatter(_)));
    }

    #[test]
    fn parse_rejects_completed_at_for_non_done_task() {
        let markdown = "---\nid: task-1\ntitle: Ship v2\nqueue: inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ntags: []\ncompleted_at: 2026-03-10T08:00:00Z\ndaily_note: null\n---\n";
        let err = parse_task_markdown(markdown).expect_err("markdown should fail validation");
        assert!(matches!(err, FormatError::CompletedAtWithoutDoneQueue));
    }
}
