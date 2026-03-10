use crate::{
    domain::{
        filter::queue_counts,
        task::{Queue, Task},
    },
    storage::repo::StoredTask,
};
use dialoguer::console::style;
use std::path::Path;

fn styled_field_label(label: &str) -> String {
    style(label).bold().cyan().to_string()
}

pub fn print_info(message: &str) {
    println!("{}", style(message).cyan());
}

pub fn print_error(message: &str) {
    eprintln!("{message}");
}

pub fn print_queue_tasks(queue: Queue, tasks: &[Task]) {
    println!("{}", style(queue.to_string()).bold().cyan());

    if tasks.is_empty() {
        println!("No tasks found");
        return;
    }

    for task in tasks {
        println!("{}  {}", style(&task.id).cyan(), task.title);
    }
}

pub fn print_dashboard(tasks: &[Task]) {
    let counts = queue_counts(tasks);
    for (queue, count) in counts {
        println!("{queue:<5} {count}");
    }

    println!();
    print_queue_tasks(
        Queue::Now,
        &tasks
            .iter()
            .filter(|task| task.queue == Queue::Now)
            .cloned()
            .collect::<Vec<_>>(),
    );
    println!();
    print_queue_tasks(
        Queue::Inbox,
        &tasks
            .iter()
            .filter(|task| task.queue == Queue::Inbox)
            .cloned()
            .collect::<Vec<_>>(),
    );
}

pub fn print_task_detail(task: &Task, path: &Path) {
    println!("{} {}", styled_field_label("ID:"), style(&task.id).cyan());
    println!(
        "{} {}",
        styled_field_label("Queue:"),
        style(task.queue.to_string()).cyan()
    );
    println!(
        "{} {}",
        styled_field_label("Path:"),
        style(path.display().to_string()).dim()
    );
    println!(
        "{} {}",
        styled_field_label("Created:"),
        style(task.created_at.to_rfc3339()).dim()
    );
    println!(
        "{} {}",
        styled_field_label("Updated:"),
        style(task.updated_at.to_rfc3339()).dim()
    );
    println!("{} {}", styled_field_label("Title:"), task.title);

    if !task.tags.is_empty() {
        println!("{} {}", styled_field_label("Tags:"), task.tags.join(", "));
    }

    if let Some(source) = &task.source {
        println!("{} {}", styled_field_label("Source:"), source);
    }

    if let Some(project) = &task.project {
        println!("{} {}", styled_field_label("Project:"), project);
    }

    if let Some(completed_at) = task.completed_at {
        println!(
            "{} {}",
            styled_field_label("Completed:"),
            style(completed_at.to_rfc3339()).dim()
        );
    }

    println!();
    println!("{}", task.body);
}

pub fn print_search_results(results: &[StoredTask]) {
    if results.is_empty() {
        println!("No tasks found");
        return;
    }

    for stored in results {
        println!(
            "[{}] {}  {}",
            stored.task.queue,
            style(&stored.task.id).cyan(),
            stored.task.title
        );
    }
}
