use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::task::{Task, TaskStatus};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskFrontmatter {
    id: String,
    created_at: DateTime<Utc>,
    status: TaskStatus,
    summary: String,
}

impl From<TaskFrontmatter> for Task {
    fn from(frontmatter: TaskFrontmatter) -> Self {
        Self {
            id: frontmatter.id,
            created_at: frontmatter.created_at,
            status: frontmatter.status,
            summary: frontmatter.summary,
            description: None,
        }
    }
}

impl From<&Task> for TaskFrontmatter {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id.clone(),
            created_at: task.created_at,
            status: task.status,
            summary: task.summary.clone(),
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
    if !parsed.body.is_empty() {
        task.description = Some(parsed.body);
    }

    Ok(task)
}

pub fn render_task_markdown(task: &Task) -> Result<String, FormatError> {
    let frontmatter = TaskFrontmatter::from(task);
    let yaml = serde_yaml::to_string(&frontmatter)?;

    let mut output = String::new();
    output.push_str(FRONTMATTER_DELIMITER);
    output.push('\n');
    output.push_str(&yaml);
    output.push_str(FRONTMATTER_DELIMITER);

    if let Some(description) = &task.description
        && !description.is_empty()
    {
        output.push('\n');
        output.push_str(description);
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{FormatError, parse_task_file, parse_task_markdown, render_task_markdown};
    use crate::domain::task::{Task, TaskStatus};

    fn sample_markdown() -> &'static str {
        "---\nid: cobalt-urial-7f3a\ncreated_at: 2026-02-20T22:15:00Z\nstatus: open\nsummary: Short task summary\n---\n# Notes\n- one\n- two"
    }

    #[test]
    fn parse_task_file_splits_frontmatter_and_body() {
        let parsed = parse_task_file(sample_markdown()).expect("task file should parse");
        assert!(parsed.frontmatter.contains("id: cobalt-urial-7f3a"));
        assert_eq!(parsed.body, "# Notes\n- one\n- two");
    }

    #[test]
    fn parse_task_file_requires_frontmatter_start_delimiter() {
        let err = parse_task_file("id: no-delimiter").expect_err("should fail without delimiter");
        assert!(matches!(err, FormatError::MissingFrontmatter));
    }

    #[test]
    fn parse_task_file_requires_frontmatter_end_delimiter() {
        let err = parse_task_file("---\nid: a").expect_err("should fail without end delimiter");
        assert!(matches!(err, FormatError::MissingFrontmatterEnd));
    }

    #[test]
    fn parse_task_markdown_maps_body_into_description() {
        let task = parse_task_markdown(sample_markdown()).expect("task should parse");
        assert_eq!(task.id, "cobalt-urial-7f3a");
        assert_eq!(task.status, TaskStatus::Open);
        assert_eq!(task.summary, "Short task summary");
        assert_eq!(task.description.as_deref(), Some("# Notes\n- one\n- two"));
    }

    #[test]
    fn parse_task_markdown_ignores_unknown_fields() {
        let markdown = "---\nid: cobalt-urial-7f3a\ncreated_at: 2026-02-20T22:15:00Z\nstatus: open\nsummary: Summary\nextra_field: keep-ignored\n---\n";
        let task = parse_task_markdown(markdown).expect("task should parse");
        assert_eq!(task.id, "cobalt-urial-7f3a");
    }

    #[test]
    fn render_task_markdown_roundtrips_core_fields() {
        let task = Task {
            id: "cobalt-urial-7f3a".to_string(),
            created_at: "2026-02-20T22:15:00Z"
                .parse()
                .expect("timestamp should parse"),
            status: TaskStatus::Closed,
            summary: "Short task summary".to_string(),
            description: Some("line 1\nline 2".to_string()),
        };

        let markdown = render_task_markdown(&task).expect("task should render");
        let parsed = parse_task_markdown(&markdown).expect("rendered markdown should parse");

        assert_eq!(parsed.id, task.id);
        assert_eq!(parsed.status, task.status);
        assert_eq!(parsed.summary, task.summary);
        assert_eq!(parsed.description, task.description);
    }

    #[test]
    fn render_omits_description_when_empty() {
        let task = Task {
            id: "a-b-cdef".to_string(),
            created_at: "2026-02-20T22:15:00Z"
                .parse()
                .expect("timestamp should parse"),
            status: TaskStatus::Open,
            summary: "Summary".to_string(),
            description: None,
        };

        let markdown = render_task_markdown(&task).expect("task should render");
        let last_line = markdown.lines().last().expect("markdown should have lines");
        assert_eq!(last_line, "---");
    }
}
