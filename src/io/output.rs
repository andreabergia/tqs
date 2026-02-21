use crate::domain::task::Task;

pub fn print_info(message: &str) {
    println!("{message}");
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

    println!(
        "{:<width$}{}SUMMARY",
        "ID",
        " ".repeat(gap),
        width = max_id_width
    );
    println!(
        "{:-<width$}{}",
        "",
        "-".repeat(max_id_width + gap + 7),
        width = max_id_width
    );

    for task in tasks {
        println!(
            "{:<width$}{}{}",
            task.id,
            " ".repeat(gap),
            task.summary,
            width = max_id_width
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

    println!(
        "{:<id_width$}{gap}{:<status_width$}{gap}{:<time_width$}{gap}SUMMARY",
        "ID",
        "STATUS",
        "CREATED AT",
        id_width = max_id_width,
        status_width = max_status_width,
        time_width = max_time_width,
        gap = " ".repeat(gap)
    );

    println!(
        "{:-<id_width$}{gap}{:-<status_width$}{gap}{:-<time_width$}{gap}{}",
        "",
        "",
        "",
        "-".repeat(7),
        id_width = max_id_width,
        status_width = max_status_width,
        time_width = max_time_width,
        gap = " ".repeat(gap)
    );

    for task in tasks {
        println!(
            "{:<id_width$}{gap}{:<status_width$}{gap}{:<time_width$}{gap}{}",
            task.id,
            task.status,
            task.created_at,
            task.summary,
            id_width = max_id_width,
            status_width = max_status_width,
            time_width = max_time_width,
            gap = " ".repeat(gap)
        );
    }
}
