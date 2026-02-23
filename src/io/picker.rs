use crate::app::app_error::AppError;
use crate::domain::filter::{ListMode, cycle_list_mode, matches_list_mode};
use crate::domain::task::{Task, TaskStatus};
use dialoguer::console::{Key, Term, style};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

struct TerminalGuard<'a> {
    term: &'a Term,
    rendered_lines: usize,
}

impl<'a> TerminalGuard<'a> {
    fn new(term: &'a Term) -> Result<Self, AppError> {
        term.hide_cursor()?;
        Ok(Self {
            term,
            rendered_lines: 0,
        })
    }

    fn set_rendered_lines(&mut self, lines: usize) {
        self.rendered_lines = lines;
    }
}

impl<'a> Drop for TerminalGuard<'a> {
    fn drop(&mut self) {
        let _ = self.term.clear_last_lines(self.rendered_lines);
        let _ = self.term.show_cursor();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TaskPickerOptions<'a> {
    pub prompt: &'a str,
    pub default_mode: ListMode,
    pub allowed_modes: &'a [ListMode],
}

#[derive(Debug, Clone)]
struct VisibleItem {
    task_index: usize,
    status: TaskStatus,
    id: String,
    summary: String,
    score: i64,
}

#[derive(Debug, Clone, Copy)]
struct RenderFrame<'a> {
    prompt: &'a str,
    mode: ListMode,
    allowed_modes: &'a [ListMode],
    search: &'a str,
    visible: &'a [VisibleItem],
    selected: Option<usize>,
    scroll: usize,
}

fn format_status_tag(status: TaskStatus) -> String {
    let label = match status {
        TaskStatus::Open => "open",
        TaskStatus::Closed => "closed",
    };
    format!("[{label:<6}]")
}

pub fn pick_task(
    tasks: &[Task],
    options: TaskPickerOptions<'_>,
) -> Result<Option<String>, AppError> {
    if tasks.is_empty() {
        return Ok(None);
    }

    let term = Term::stderr();
    if !term.is_term() {
        return Err(AppError::NoTty);
    }

    let allowed_modes = sanitize_allowed_modes(options.allowed_modes, options.default_mode);
    let mut mode = if allowed_modes.contains(&options.default_mode) {
        options.default_mode
    } else {
        allowed_modes[0]
    };

    let matcher = SkimMatcherV2::default();
    let mut search = String::new();
    let mut selected = Some(0usize);
    let mut scroll = 0usize;
    let mut rendered_lines = 0usize;

    let mut guard = TerminalGuard::new(&term)?;

    loop {
        let visible = build_visible_items(tasks, mode, &search, &matcher);
        sync_selection(&visible, &mut selected, &mut scroll);
        let frame = RenderFrame {
            prompt: options.prompt,
            mode,
            allowed_modes: &allowed_modes,
            search: &search,
            visible: &visible,
            selected,
            scroll,
        };
        rendered_lines = render_picker(&term, frame, rendered_lines)?;
        guard.set_rendered_lines(rendered_lines);

        match term.read_key()? {
            Key::Escape => break Ok(None),
            Key::Tab => {
                mode = cycle_list_mode(mode, &allowed_modes, false);
                selected = Some(0);
                scroll = 0;
            }
            Key::BackTab => {
                mode = cycle_list_mode(mode, &allowed_modes, true);
                selected = Some(0);
                scroll = 0;
            }
            Key::ArrowUp => {
                move_selection_up(&visible, &mut selected, &mut scroll);
            }
            Key::ArrowDown => {
                move_selection_down(&visible, &mut selected, &mut scroll);
            }
            Key::Backspace => {
                search.pop();
                selected = Some(0);
                scroll = 0;
            }
            Key::Enter => {
                if let Some(sel) = selected.and_then(|idx| visible.get(idx)) {
                    break Ok(Some(tasks[sel.task_index].id.clone()));
                }
            }
            Key::Char(ch) if !ch.is_ascii_control() => {
                search.push(ch);
                selected = Some(0);
                scroll = 0;
            }
            _ => {}
        }
    }
}

fn sanitize_allowed_modes(allowed: &[ListMode], default_mode: ListMode) -> Vec<ListMode> {
    let mut modes = Vec::new();

    for mode in allowed.iter().copied() {
        if !modes.contains(&mode) {
            modes.push(mode);
        }
    }

    if modes.is_empty() {
        modes.push(default_mode);
    }

    if !modes.contains(&default_mode) {
        modes.insert(0, default_mode);
    }

    modes
}

fn build_visible_items(
    tasks: &[Task],
    mode: ListMode,
    search: &str,
    matcher: &SkimMatcherV2,
) -> Vec<VisibleItem> {
    let mut visible = tasks
        .iter()
        .enumerate()
        .filter(|(_, task)| matches_list_mode(task, mode))
        .filter_map(|(idx, task)| {
            let display = format!("[{}] {} - {}", task.status, task.id, task.summary);
            let score = if search.is_empty() {
                Some(0)
            } else {
                matcher.fuzzy_match(&display, search)
            }?;

            Some(VisibleItem {
                task_index: idx,
                status: task.status,
                id: task.id.clone(),
                summary: task.summary.clone(),
                score,
            })
        })
        .collect::<Vec<_>>();

    visible.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.task_index.cmp(&b.task_index))
    });
    visible
}

fn sync_selection(visible: &[VisibleItem], selected: &mut Option<usize>, scroll: &mut usize) {
    if visible.is_empty() {
        *selected = None;
        *scroll = 0;
        return;
    }

    let idx = selected.unwrap_or(0).min(visible.len() - 1);
    *selected = Some(idx);
}

fn render_picker(
    term: &Term,
    frame: RenderFrame<'_>,
    prev_rendered_lines: usize,
) -> Result<usize, AppError> {
    if prev_rendered_lines > 0 {
        term.clear_last_lines(prev_rendered_lines)?;
    }

    let mode_list = frame
        .allowed_modes
        .iter()
        .map(|m| {
            if *m == frame.mode {
                format!(
                    "{}",
                    style(format!(" {} ", m.label())).bold().black().on_cyan()
                )
            } else {
                format!("{}", style(m.label()).cyan())
            }
        })
        .collect::<Vec<_>>()
        .join("/");

    let prompt_line = if frame.search.is_empty() {
        format!("{} [{mode_list}]", frame.prompt)
    } else {
        format!("{} [{mode_list}] search: {}", frame.prompt, frame.search)
    };
    let hint_line = format!(
        "{}",
        style("Tab/Shift-Tab: filter  Up/Down: navigate  Enter: select  Esc: cancel").cyan()
    );

    term.write_line(&prompt_line)?;
    term.write_line(&hint_line)?;

    let rows = term.size().0 as usize;
    let max_items = rows.saturating_sub(2).max(1);

    let mut line_count = 2;
    if frame.visible.is_empty() {
        term.write_line(&format!("{}", style("  (no matching tasks)").yellow()))?;
        return Ok(line_count + 1);
    }

    let sel = frame.selected.unwrap_or(0);
    let mut start = frame.scroll.min(sel);
    if sel >= start + max_items {
        start = sel + 1 - max_items;
    }

    for (idx, item) in frame.visible.iter().enumerate().skip(start).take(max_items) {
        let is_selected = Some(idx) == frame.selected;

        let prefix = if is_selected {
            format!("{}", style(">").bold().black().on_cyan())
        } else {
            " ".to_string()
        };

        let line = if is_selected {
            let line_content = format!(
                "{} {} - {}",
                format_status_tag(item.status),
                item.id,
                item.summary
            );
            style(line_content).bold().black().on_cyan()
        } else {
            let status_tag = format_status_tag(item.status);
            let status = match item.status {
                TaskStatus::Open => style(&status_tag).green(),
                TaskStatus::Closed => style(&status_tag).yellow(),
            };
            let summary_style = style(&item.summary);
            let line_content = format!("{status} {} - {summary_style}", item.id);
            style(line_content)
        };

        term.write_line(&format!("{prefix} {line}"))?;
        line_count += 1;
    }

    Ok(line_count)
}

fn move_selection_up(visible: &[VisibleItem], selected: &mut Option<usize>, scroll: &mut usize) {
    if visible.is_empty() {
        return;
    }

    let next = match *selected {
        Some(0) | None => visible.len() - 1,
        Some(idx) => idx - 1,
    };

    *selected = Some(next);
    *scroll = (*scroll).min(next);
}

fn move_selection_down(visible: &[VisibleItem], selected: &mut Option<usize>, scroll: &mut usize) {
    if visible.is_empty() {
        return;
    }

    let next = match *selected {
        None => 0,
        Some(idx) => (idx + 1) % visible.len(),
    };

    *selected = Some(next);
    if next == 0 {
        *scroll = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::{build_visible_items, format_status_tag, sanitize_allowed_modes, sync_selection};
    use crate::domain::filter::ListMode;
    use crate::domain::task::{Task, TaskStatus};
    use chrono::{DateTime, Utc};
    use fuzzy_matcher::skim::SkimMatcherV2;

    fn created_at() -> DateTime<Utc> {
        "2026-02-20T22:15:00Z"
            .parse()
            .expect("timestamp literal should parse")
    }

    fn task(id: &str, status: TaskStatus, summary: &str) -> Task {
        Task {
            id: id.to_string(),
            created_at: created_at(),
            status,
            summary: summary.to_string(),
            description: None,
        }
    }

    #[test]
    fn sanitize_allowed_modes_keeps_uniques_and_default() {
        let modes = sanitize_allowed_modes(&[ListMode::All, ListMode::All], ListMode::Open);
        assert_eq!(modes, vec![ListMode::Open, ListMode::All]);
    }

    #[test]
    fn build_visible_items_filters_by_status_and_search() {
        let tasks = vec![
            task("alpha", TaskStatus::Open, "Write docs"),
            task("beta", TaskStatus::Closed, "Fix parser"),
        ];
        let matcher = SkimMatcherV2::default();

        let open = build_visible_items(&tasks, ListMode::Open, "", &matcher);
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].task_index, 0);

        let closed_search = build_visible_items(&tasks, ListMode::Closed, "parser", &matcher);
        assert_eq!(closed_search.len(), 1);
        assert_eq!(closed_search[0].task_index, 1);
    }

    #[test]
    fn sync_selection_clears_when_list_empty() {
        let mut selected = Some(3);
        let mut scroll = 5;

        sync_selection(&[], &mut selected, &mut scroll);

        assert_eq!(selected, None);
        assert_eq!(scroll, 0);
    }

    #[test]
    fn status_tags_use_equal_width() {
        let open = format_status_tag(TaskStatus::Open);
        let closed = format_status_tag(TaskStatus::Closed);

        assert_eq!(open, "[open  ]");
        assert_eq!(closed, "[closed]");
        assert_eq!(open.len(), closed.len());
    }
}
