use super::task::Task;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMode {
    Open,
    Closed,
    All,
}

impl ListMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::All => "all",
        }
    }
}

pub fn matches_list_mode(task: &Task, mode: ListMode) -> bool {
    match mode {
        ListMode::All => true,
        ListMode::Open => task.status.is_open(),
        ListMode::Closed => task.status.is_closed(),
    }
}

pub fn cycle_list_mode(current: ListMode, allowed: &[ListMode], reverse: bool) -> ListMode {
    if allowed.is_empty() {
        return current;
    }

    let current_idx = allowed
        .iter()
        .position(|mode| *mode == current)
        .unwrap_or(0);
    let next_idx = if reverse {
        (current_idx + allowed.len() - 1) % allowed.len()
    } else {
        (current_idx + 1) % allowed.len()
    };

    allowed[next_idx]
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
    use super::{ListMode, cycle_list_mode, matches_keywords, matches_list_mode};
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

    #[test]
    fn matches_list_mode_uses_task_status() {
        let mut target = task("Write docs", None);
        target.status = TaskStatus::Closed;

        assert!(matches_list_mode(&target, ListMode::All));
        assert!(!matches_list_mode(&target, ListMode::Open));
        assert!(matches_list_mode(&target, ListMode::Closed));
    }

    #[test]
    fn cycle_list_mode_moves_forward_and_backward() {
        let allowed = [ListMode::Open, ListMode::All];
        assert_eq!(
            cycle_list_mode(ListMode::Open, &allowed, false),
            ListMode::All
        );
        assert_eq!(
            cycle_list_mode(ListMode::All, &allowed, false),
            ListMode::Open
        );
        assert_eq!(
            cycle_list_mode(ListMode::Open, &allowed, true),
            ListMode::All
        );
    }
}
