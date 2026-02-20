use super::task::Task;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMode {
    Open,
    Closed,
    All,
}

pub fn matches_keywords(task: &Task, keywords: &[String]) -> bool {
    if keywords.is_empty() {
        return true;
    }

    let description = task.description.as_deref().unwrap_or_default();
    let haystack = format!("{} {} {}", task.id, task.summary, description).to_lowercase();

    keywords
        .iter()
        .all(|kw| haystack.contains(&kw.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::matches_keywords;
    use crate::domain::task::{Task, TaskStatus};
    use chrono::{DateTime, Utc};

    fn task(summary: &str, description: Option<&str>) -> Task {
        Task {
            id: "cobalt-urial-7f3a".to_string(),
            created_at: "2026-02-20T22:15:00Z"
                .parse::<DateTime<Utc>>()
                .expect("timestamp should parse"),
            status: TaskStatus::Open,
            summary: summary.to_string(),
            description: description.map(str::to_string),
        }
    }

    #[test]
    fn empty_keywords_match_every_task() {
        assert!(matches_keywords(&task("Write docs", None), &[]));
    }

    #[test]
    fn keywords_use_case_insensitive_and_semantics() {
        let target = task("Write Parser", Some("Handle Markdown frontmatter"));
        let ok = vec!["write".to_string(), "frontmatter".to_string()];
        let not_ok = vec!["write".to_string(), "missing".to_string()];

        assert!(matches_keywords(&target, &ok));
        assert!(!matches_keywords(&target, &not_ok));
    }
}
