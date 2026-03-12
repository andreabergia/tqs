use crate::{
    domain::{
        filter::queue_counts,
        task::{Queue, Task},
    },
    storage::config::{ConfigInspection, ResolvedConfig},
    storage::doctor::{DiagnosticSeverity, DoctorReport},
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

pub fn print_config(config: &ResolvedConfig) {
    if let Some(path) = &config.obsidian_vault_dir {
        println!("obsidian_vault_dir = {}", path.display());
    }

    println!("tasks_root = {}", config.tasks_root.display());
    println!("state_dir = {}", config.state_dir.display());

    match &config.daily_notes_dir {
        Some(path) => println!("daily_notes_dir = {}", path.display()),
        None => println!("daily_notes_dir = <unset>"),
    }

    println!("queue.inbox = {}", config.queue_dirs.inbox);
    println!("queue.now = {}", config.queue_dirs.now);
    println!("queue.next = {}", config.queue_dirs.next);
    println!("queue.later = {}", config.queue_dirs.later);
    println!("queue.done = {}", config.queue_dirs.done);
}

pub fn print_config_inspection(inspection: &ConfigInspection) {
    match &inspection.config_path {
        Some(path) => println!("config_path = {}", path.display()),
        None => println!("config_path = <unavailable>"),
    }

    println!(
        "config_file = {}",
        if inspection.file_exists {
            "present"
        } else {
            "missing"
        }
    );

    match &inspection.explicit_root {
        Some(path) => println!("root_cli = {}", path.display()),
        None => println!("root_cli = <unset>"),
    }

    match &inspection.env_root {
        Some(path) => println!("root_env = {}", path.display()),
        None => println!("root_env = <unset>"),
    }

    match &inspection.resolved {
        Some(config) => {
            println!();
            print_config(config);
        }
        None => {
            println!("tasks_root = <unset>");
            println!();
            println!(
                "{}",
                crate::storage::config::starter_config(inspection.config_path.as_deref())
            );
        }
    }
}

pub fn print_doctor_report(report: &DoctorReport) {
    for diagnostic in &report.diagnostics {
        let label = match diagnostic.severity {
            DiagnosticSeverity::Ok => style("ok").green().bold().to_string(),
            DiagnosticSeverity::Warning => style("warn").yellow().bold().to_string(),
            DiagnosticSeverity::Error => style("error").red().bold().to_string(),
        };
        println!("[{label}] {}: {}", diagnostic.scope, diagnostic.message);
    }

    println!(
        "summary: {} ok, {} warning(s), {} error(s)",
        report.ok_count(),
        report.warning_count(),
        report.error_count()
    );
}
