use super::task::{Queue, Task};

pub fn matches_query(task: &Task, query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return true;
    }

    let tags = task.tags.join(" ");
    let source = task.source.as_deref().unwrap_or_default();
    let project = task.project.as_deref().unwrap_or_default();
    let haystack = format!(
        "{} {} {} {} {} {}",
        task.id, task.title, task.body, tags, source, project
    )
    .to_ascii_lowercase();

    haystack.contains(&query)
}

pub fn queue_counts(tasks: &[Task]) -> Vec<(Queue, usize)> {
    Queue::all()
        .iter()
        .copied()
        .map(|queue| {
            let count = tasks.iter().filter(|task| task.queue == queue).count();
            (queue, count)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{matches_query, queue_counts};
    use crate::domain::task::{Queue, Task};

    fn task() -> Task {
        let mut task = Task::new(
            "20260309-aws-billing",
            "Reply to AWS billing alert",
            "2026-03-09T10:34:12Z"
                .parse()
                .expect("timestamp should parse"),
        );
        task.tags = vec!["aws".to_string(), "finance".to_string()];
        task.project = Some("platform-costs".to_string());
        task.body = "Check cost explorer".to_string();
        task
    }

    #[test]
    fn matches_query_uses_metadata_and_body() {
        let task = task();
        assert!(matches_query(&task, "billing"));
        assert!(matches_query(&task, "cost explorer"));
        assert!(matches_query(&task, "platform-costs"));
        assert!(!matches_query(&task, "missing"));
    }

    #[test]
    fn queue_counts_cover_every_builtin_queue() {
        let mut task = task();
        task.queue = Queue::Now;
        let counts = queue_counts(&[task]);

        assert_eq!(counts.len(), Queue::all().len());
        assert_eq!(
            counts
                .iter()
                .find(|(queue, _)| *queue == Queue::Now)
                .expect("now queue should exist")
                .1,
            1
        );
    }
}
