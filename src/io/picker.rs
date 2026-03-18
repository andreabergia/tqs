use crate::app::app_error::AppError;
use crate::domain::task::Queue;
use crate::storage::repo::StoredTask;
use dialoguer::console::{Key, Term, style};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

struct TerminalGuard<'a> {
    term: &'a Term,
    rendered_lines: usize,
    #[cfg(unix)]
    saved_termios: Option<libc::termios>,
    #[cfg(unix)]
    tty_fd: Option<std::os::unix::io::RawFd>,
}

impl<'a> TerminalGuard<'a> {
    fn new(term: &'a Term) -> Result<Self, AppError> {
        #[cfg(unix)]
        let (saved_termios, tty_fd) = {
            let fd = unsafe { libc::open(c"/dev/tty".as_ptr(), libc::O_RDWR) };
            if fd >= 0 {
                let mut termios = unsafe { std::mem::zeroed::<libc::termios>() };
                if unsafe { libc::tcgetattr(fd, &mut termios) } == 0 {
                    (Some(termios), Some(fd))
                } else {
                    unsafe { libc::close(fd) };
                    (None, None)
                }
            } else {
                (None, None)
            }
        };

        term.hide_cursor()?;
        Ok(Self {
            term,
            rendered_lines: 0,
            #[cfg(unix)]
            saved_termios,
            #[cfg(unix)]
            tty_fd,
        })
    }

    fn set_rendered_lines(&mut self, rendered_lines: usize) {
        self.rendered_lines = rendered_lines;
    }
}

impl Drop for TerminalGuard<'_> {
    fn drop(&mut self) {
        let _ = self.term.clear_last_lines(self.rendered_lines);
        let _ = self.term.show_cursor();

        #[cfg(unix)]
        if let (Some(termios), Some(fd)) = (self.saved_termios.as_ref(), self.tty_fd) {
            unsafe {
                libc::tcsetattr(fd, libc::TCSANOW, termios);
                libc::close(fd);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TaskPickerOptions<'a> {
    pub prompt: &'a str,
}

#[derive(Debug, Clone)]
struct VisibleItem {
    task_index: usize,
    score: i64,
}

#[derive(Debug, Clone, Copy)]
struct RenderState<'a> {
    prompt: &'a str,
    search: &'a str,
    queue_filter: Option<Queue>,
    selected: Option<usize>,
    scroll: usize,
    prev_rendered_lines: usize,
}

pub fn pick_task(
    tasks: &[StoredTask],
    options: TaskPickerOptions<'_>,
) -> Result<Option<usize>, AppError> {
    if tasks.is_empty() {
        return Ok(None);
    }

    let term = Term::stderr();
    if !term.is_term() || cfg!(test) {
        return Err(AppError::NoTty);
    }

    let matcher = SkimMatcherV2::default();
    let mut search = String::new();
    let mut queue_filter: Option<Queue> = None;
    let mut selected = Some(0usize);
    let mut scroll = 0usize;
    let mut rendered_lines = 0usize;
    let mut guard = TerminalGuard::new(&term)?;

    loop {
        let visible = build_visible_items(tasks, &search, queue_filter, &matcher);
        sync_selection(&visible, &mut selected, &mut scroll);
        rendered_lines = render_picker(
            &term,
            RenderState {
                prompt: options.prompt,
                search: &search,
                queue_filter,
                selected,
                scroll,
                prev_rendered_lines: rendered_lines,
            },
            tasks,
            &visible,
        )?;
        guard.set_rendered_lines(rendered_lines);

        match term.read_key()? {
            Key::Escape | Key::Char('\x03') => break Ok(None),
            Key::ArrowUp => move_selection_up(&visible, &mut selected, &mut scroll),
            Key::ArrowDown => move_selection_down(&visible, &mut selected, &mut scroll),
            Key::Tab => {
                queue_filter = next_queue_filter(queue_filter);
                selected = Some(0);
                scroll = 0;
            }
            Key::BackTab => {
                queue_filter = prev_queue_filter(queue_filter);
                selected = Some(0);
                scroll = 0;
            }
            Key::Backspace => {
                search.pop();
                selected = Some(0);
                scroll = 0;
            }
            Key::Enter => {
                if let Some(choice) = selected.and_then(|index| visible.get(index)) {
                    break Ok(Some(choice.task_index));
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

fn next_queue_filter(current: Option<Queue>) -> Option<Queue> {
    match current {
        None => Some(Queue::Inbox),
        Some(Queue::Inbox) => Some(Queue::Now),
        Some(Queue::Now) => Some(Queue::Next),
        Some(Queue::Next) => Some(Queue::Later),
        Some(Queue::Later) => Some(Queue::Done),
        Some(Queue::Done) => None,
    }
}

fn prev_queue_filter(current: Option<Queue>) -> Option<Queue> {
    match current {
        None => Some(Queue::Done),
        Some(Queue::Done) => Some(Queue::Later),
        Some(Queue::Later) => Some(Queue::Next),
        Some(Queue::Next) => Some(Queue::Now),
        Some(Queue::Now) => Some(Queue::Inbox),
        Some(Queue::Inbox) => None,
    }
}

fn build_visible_items(
    tasks: &[StoredTask],
    search: &str,
    queue_filter: Option<Queue>,
    matcher: &SkimMatcherV2,
) -> Vec<VisibleItem> {
    let mut visible = tasks
        .iter()
        .enumerate()
        .filter_map(|(index, stored)| {
            if let Some(q) = queue_filter
                && stored.task.queue != q
            {
                return None;
            }

            let display = format!(
                "{} {} - {}",
                stored.task.queue, stored.task.id, stored.task.title
            );
            let score = if search.is_empty() {
                Some(0)
            } else {
                matcher.fuzzy_match(&display, search)
            }?;

            Some(VisibleItem {
                task_index: index,
                score,
            })
        })
        .collect::<Vec<_>>();

    visible.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.task_index.cmp(&right.task_index))
    });
    visible
}

fn sync_selection(visible: &[VisibleItem], selected: &mut Option<usize>, scroll: &mut usize) {
    if visible.is_empty() {
        *selected = None;
        *scroll = 0;
        return;
    }

    *selected = Some(selected.unwrap_or(0).min(visible.len() - 1));
}

fn render_picker(
    term: &Term,
    state: RenderState<'_>,
    tasks: &[StoredTask],
    visible: &[VisibleItem],
) -> Result<usize, AppError> {
    if state.prev_rendered_lines > 0 {
        term.clear_last_lines(state.prev_rendered_lines)?;
    }

    let mut prompt_line = state.prompt.to_string();
    if let Some(q) = state.queue_filter {
        prompt_line.push_str(&format!("  queue: {}", style(q).magenta()));
    }
    if !state.search.is_empty() {
        prompt_line.push_str(&format!("  search: {}", state.search));
    }

    term.write_line(&prompt_line)?;
    term.write_line(&format!(
        "{}",
        style("Up/Down: navigate  Tab/Shift-Tab: filter by queue  Enter: select  Esc: cancel")
            .cyan()
    ))?;

    let rows = term.size().0 as usize;
    let max_items = rows.saturating_sub(2).max(1);
    let mut line_count = 2;

    if visible.is_empty() {
        term.write_line(&format!("{}", style("  (no matching tasks)").yellow()))?;
        return Ok(line_count + 1);
    }

    let selected_index = state.selected.unwrap_or(0);
    let mut start = state.scroll.min(selected_index);
    if selected_index >= start + max_items {
        start = selected_index + 1 - max_items;
    }

    for (visible_index, item) in visible.iter().enumerate().skip(start).take(max_items) {
        let is_selected = Some(visible_index) == state.selected;
        let stored = &tasks[item.task_index];
        let queue_str = format!("{:<5}", stored.task.queue);
        let rest = format!("  {} - {}", stored.task.id, stored.task.title);

        if is_selected {
            term.write_line(&format!(
                "{} {}{}",
                style(">").bold().black().on_cyan(),
                style(queue_str).bold().black().on_cyan(),
                style(rest).bold().black().on_cyan()
            ))?;
        } else {
            term.write_line(&format!("  {}{}", style(queue_str).magenta(), rest))?;
        }

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
        Some(index) => index - 1,
    };

    *selected = Some(next);
    *scroll = (*scroll).min(next);
}

fn move_selection_down(visible: &[VisibleItem], selected: &mut Option<usize>, _scroll: &mut usize) {
    if visible.is_empty() {
        return;
    }

    let next = match *selected {
        Some(index) if index + 1 < visible.len() => index + 1,
        _ => 0,
    };

    *selected = Some(next);
}

#[cfg(test)]
mod tests {
    use super::{build_visible_items, sync_selection};
    use crate::{domain::task::Task, storage::repo::StoredTask};
    use fuzzy_matcher::skim::SkimMatcherV2;
    use std::path::PathBuf;

    fn stored_task(id: &str, title: &str) -> StoredTask {
        StoredTask {
            task: Task::new(
                id,
                title,
                "2026-03-09T10:34:12Z"
                    .parse()
                    .expect("timestamp should parse"),
            ),
            path: PathBuf::from(format!("/tmp/{id}.md")),
        }
    }

    #[test]
    fn build_visible_items_uses_fuzzy_search() {
        let matcher = SkimMatcherV2::default();
        let items = vec![stored_task("task-1", "Reply to AWS billing alert")];
        let visible = build_visible_items(&items, "aws", None, &matcher);
        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn sync_selection_clears_when_empty() {
        let mut selected = Some(1);
        let mut scroll = 2;
        sync_selection(&[], &mut selected, &mut scroll);
        assert_eq!(selected, None);
        assert_eq!(scroll, 0);
    }
}
