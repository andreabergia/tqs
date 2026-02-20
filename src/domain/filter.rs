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
