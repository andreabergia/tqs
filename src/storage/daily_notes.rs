use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::NaiveDate;

use crate::{app::app_error::AppError, domain::task::Task};

const COMPLETED_TASKS_HEADING: &str = "## Completed Tasks";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyNoteUpdate {
    pub note_name: String,
    pub note_path: PathBuf,
    pub appended: bool,
}

pub fn append_completion(
    daily_notes_dir: &Path,
    note_date: NaiveDate,
    task: &Task,
) -> Result<DailyNoteUpdate, AppError> {
    fs::create_dir_all(daily_notes_dir)?;

    let note_name = format!("{}.md", note_date.format("%F"));
    let note_path = daily_notes_dir.join(&note_name);
    let entry = completion_entry(task);
    let existing = if note_path.exists() {
        fs::read_to_string(&note_path)?
    } else {
        String::new()
    };

    if existing
        .lines()
        .any(|line| is_completion_entry_for_task(line, &task.id))
    {
        return Ok(DailyNoteUpdate {
            note_name,
            note_path,
            appended: false,
        });
    }

    let updated = append_entry_to_note(&existing, &entry);
    fs::write(&note_path, updated)?;

    Ok(DailyNoteUpdate {
        note_name,
        note_path,
        appended: true,
    })
}

fn completion_entry(task: &Task) -> String {
    format!("- [x] [[Tasks/done/{}|{}]]", task.id, task.title)
}

fn is_completion_entry_for_task(line: &str, task_id: &str) -> bool {
    let wiki_link = format!("- [x] [[Tasks/done/{task_id}|");
    let plain_text = format!(" ({task_id})");

    line == wiki_link || line.starts_with(&wiki_link) || line.ends_with(&plain_text)
}

fn append_entry_to_note(existing: &str, entry: &str) -> String {
    if existing.trim().is_empty() {
        return format!("{COMPLETED_TASKS_HEADING}\n\n{entry}\n");
    }

    let mut lines = existing.lines().map(str::to_string).collect::<Vec<_>>();
    let heading_index = lines
        .iter()
        .position(|line| line.trim() == COMPLETED_TASKS_HEADING);

    match heading_index {
        Some(index) => {
            let mut insert_at = lines[index + 1..]
                .iter()
                .position(|line| is_section_heading(line))
                .map_or(lines.len(), |offset| index + 1 + offset);

            while insert_at > index + 1 && lines[insert_at - 1].is_empty() {
                insert_at -= 1;
            }

            lines.insert(insert_at, entry.to_string());
            normalize_spacing_around_insert(&mut lines, index, insert_at);
            render_lines(&lines)
        }
        None => {
            let mut rendered = existing.trim_end_matches('\n').to_string();
            rendered.push_str("\n\n");
            rendered.push_str(COMPLETED_TASKS_HEADING);
            rendered.push_str("\n\n");
            rendered.push_str(entry);
            rendered.push('\n');
            rendered
        }
    }
}

fn is_section_heading(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('#') && trimmed.chars().take_while(|char| *char == '#').count() >= 2
}

fn normalize_spacing_around_insert(
    lines: &mut Vec<String>,
    heading_index: usize,
    insert_at: usize,
) {
    if insert_at == heading_index + 1 {
        lines.insert(insert_at, String::new());
    }

    let entry_index = if insert_at == heading_index + 1 {
        insert_at + 1
    } else {
        insert_at
    };

    if entry_index + 1 < lines.len() && !lines[entry_index + 1].is_empty() {
        lines.insert(entry_index + 1, String::new());
    }
}

fn render_lines(lines: &[String]) -> String {
    let mut rendered = lines.join("\n");
    rendered.push('\n');
    rendered
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use tempfile::TempDir;

    use super::{COMPLETED_TASKS_HEADING, append_completion};
    use crate::domain::task::Task;

    fn task() -> Task {
        Task::new(
            "task-1",
            "Ship v2",
            "2026-03-10T08:00:00Z"
                .parse()
                .expect("timestamp should parse"),
        )
    }

    #[test]
    fn creates_note_with_completed_tasks_section_when_missing() {
        let temp = TempDir::new().expect("temp dir should exist");
        let update = append_completion(
            temp.path(),
            NaiveDate::from_ymd_opt(2026, 3, 10).expect("date should exist"),
            &task(),
        )
        .expect("append should succeed");

        assert!(update.appended);
        let note = std::fs::read_to_string(update.note_path).expect("note should exist");
        assert_eq!(
            note,
            format!("{COMPLETED_TASKS_HEADING}\n\n- [x] [[Tasks/done/task-1|Ship v2]]\n")
        );
    }

    #[test]
    fn appends_inside_existing_completed_tasks_section() {
        let temp = TempDir::new().expect("temp dir should exist");
        let note_path = temp.path().join("2026-03-10.md");
        std::fs::write(
            &note_path,
            "# Daily\n\n## Completed Tasks\n\n- [x] [[Tasks/done/task-0|Existing]]\n\n## Notes\n\nStuff\n",
        )
        .expect("note should be written");

        append_completion(
            temp.path(),
            NaiveDate::from_ymd_opt(2026, 3, 10).expect("date should exist"),
            &task(),
        )
        .expect("append should succeed");

        let note = std::fs::read_to_string(note_path).expect("note should exist");
        assert_eq!(
            note,
            "# Daily\n\n## Completed Tasks\n\n- [x] [[Tasks/done/task-0|Existing]]\n- [x] [[Tasks/done/task-1|Ship v2]]\n\n## Notes\n\nStuff\n"
        );
    }

    #[test]
    fn skips_duplicate_completion_entries() {
        let temp = TempDir::new().expect("temp dir should exist");
        let note_path = temp.path().join("2026-03-10.md");
        std::fs::write(
            &note_path,
            "## Completed Tasks\n\n- [x] [[Tasks/done/task-1|Ship v2]]\n",
        )
        .expect("note should be written");

        let update = append_completion(
            temp.path(),
            NaiveDate::from_ymd_opt(2026, 3, 10).expect("date should exist"),
            &task(),
        )
        .expect("append should succeed");

        assert!(!update.appended);
        let note = std::fs::read_to_string(note_path).expect("note should exist");
        assert_eq!(
            note,
            "## Completed Tasks\n\n- [x] [[Tasks/done/task-1|Ship v2]]\n"
        );
    }

    #[test]
    fn skips_duplicate_completion_entries_when_plain_text_format_exists() {
        let temp = TempDir::new().expect("temp dir should exist");
        let note_path = temp.path().join("2026-03-10.md");
        std::fs::write(&note_path, "## Completed Tasks\n\n- [x] Ship v2 (task-1)\n")
            .expect("note should be written");

        let update = append_completion(
            temp.path(),
            NaiveDate::from_ymd_opt(2026, 3, 10).expect("date should exist"),
            &task(),
        )
        .expect("append should succeed");

        assert!(!update.appended);
        let note = std::fs::read_to_string(note_path).expect("note should exist");
        assert_eq!(note, "## Completed Tasks\n\n- [x] Ship v2 (task-1)\n");
    }
}
