use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use super::panel_border_style;
use crate::domain::task::Task;
use crate::tui::app_state::QueueFilter;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    filter: QueueFilter,
    tasks: &[&Task],
    selected: Option<usize>,
    focused: bool,
) {
    let title = match filter {
        QueueFilter::Single(queue) => format!(" Tasks in queue {queue} ({}) ", tasks.len()),
        QueueFilter::All => format!(" All tasks ({}) ", tasks.len()),
    };

    let show_queue_tag = matches!(filter, QueueFilter::All);

    let items: Vec<ListItem> = tasks
        .iter()
        .map(|task| {
            let mut spans = Vec::new();
            if show_queue_tag {
                spans.push(Span::styled(
                    format!("[{:<5}] ", task.queue),
                    Style::default().fg(Color::Magenta),
                ));
            }
            spans.push(Span::styled(
                format!("{:<8}", task.id),
                Style::default().fg(Color::Cyan),
            ));
            spans.push(Span::raw(&task.title));
            ListItem::new(Line::from(spans))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(panel_border_style(focused));

    let highlight_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .bg(Color::DarkGray);

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style)
        .highlight_symbol("> ");

    let mut state = ListState::default().with_selected(selected);
    frame.render_stateful_widget(list, area, &mut state);
}
