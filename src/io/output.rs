use crate::domain::task::{Task, TaskStatus};
use dialoguer::console::style;

fn styled_status(status: TaskStatus) -> String {
    match status {
        TaskStatus::Open => style("open").green().to_string(),
        TaskStatus::Closed => style("closed").yellow().to_string(),
    }
}

fn styled_field_label(label: &str) -> String {
    style(label).bold().cyan().to_string()
}

pub fn print_info(message: &str) {
    println!("{}", style(message).cyan());
}

pub fn print_error(message: &str) {
    eprintln!("{message}");
}

pub fn print_tasks_simple(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("No tasks found");
        return;
    }

    let max_id_width = tasks.iter().map(|t| t.id.len()).max().unwrap_or(0);
    let gap = 2;
    let gap_str = " ".repeat(gap);
    let header_id = format!("{:<width$}", "ID", width = max_id_width);
    let header_line = format!(
        "{}{}{}",
        style(header_id).bold().cyan(),
        gap_str,
        style("SUMMARY").bold().cyan()
    );
    let separator = format!(
        "{}{}{}",
        "-".repeat(max_id_width),
        " ".repeat(gap),
        "-".repeat(7)
    );

    println!("{header_line}");
    println!("{}", style(separator).dim());

    for task in tasks {
        let padded_id = format!("{:<width$}", task.id, width = max_id_width);
        println!(
            "{}{}{}",
            style(padded_id).cyan(),
            " ".repeat(gap),
            task.summary,
        );
    }
}

pub fn print_tasks_verbose(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("No tasks found");
        return;
    }

    let max_id_width = tasks.iter().map(|t| t.id.len()).max().unwrap_or(0);
    let max_status_width = tasks
        .iter()
        .map(|t| t.status.to_string().len())
        .max()
        .unwrap_or(0);
    let max_time_width = tasks
        .iter()
        .map(|t| t.created_at.to_string().len())
        .max()
        .unwrap_or(0);
    let gap = 2;
    let gap_str = " ".repeat(gap);
    let header_id = format!("{:<id_width$}", "ID", id_width = max_id_width);
    let header_status = format!(
        "{:<status_width$}",
        "STATUS",
        status_width = max_status_width
    );
    let header_time = format!("{:<time_width$}", "CREATED AT", time_width = max_time_width);
    let header_line = format!(
        "{}{gap}{}{gap}{}{gap}{}",
        style(header_id).bold().cyan(),
        style(header_status).bold().cyan(),
        style(header_time).bold().cyan(),
        style("SUMMARY").bold().cyan(),
        gap = gap_str
    );
    let separator = format!(
        "{}{gap}{}{gap}{}{gap}{}",
        "-".repeat(max_id_width),
        "-".repeat(max_status_width),
        "-".repeat(max_time_width),
        "-".repeat(7),
        gap = " ".repeat(gap)
    );

    println!("{header_line}");
    println!("{}", style(separator).dim());

    for task in tasks {
        let padded_id = format!("{:<id_width$}", task.id, id_width = max_id_width);
        let padded_status = format!(
            "{:<status_width$}",
            task.status,
            status_width = max_status_width
        );
        let padded_time = format!(
            "{:<time_width$}",
            task.created_at,
            time_width = max_time_width
        );
        println!(
            "{}{gap}{}{gap}{}{gap}{}",
            style(padded_id).cyan(),
            match task.status {
                TaskStatus::Open => style(padded_status).green(),
                TaskStatus::Closed => style(padded_status).yellow(),
            },
            style(padded_time).dim(),
            task.summary,
            gap = " ".repeat(gap)
        );
    }
}

pub fn print_task_detail(task: &Task) {
    println!("{} {}", styled_field_label("ID:"), style(&task.id).cyan());
    println!(
        "{} {}",
        styled_field_label("Status:"),
        styled_status(task.status)
    );
    println!(
        "{} {}",
        styled_field_label("Created at:"),
        style(task.created_at.to_string()).dim()
    );
    println!("{} {}", styled_field_label("Summary:"), task.summary);

    if let Some(description) = &task.description {
        println!();
        println!("{description}");
    }
}
