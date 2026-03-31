use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::domain::task::{Queue, Task};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    queue: Queue,
    tasks: &[&Task],
    state: &mut ListState,
    focused: bool,
) {
    let title = format!(" {} ({}) ", queue, tasks.len());

    let items: Vec<ListItem> = tasks
        .iter()
        .map(|task| {
            let line = Line::from(vec![
                Span::styled(format!("{:<8}", task.id), Style::default().fg(Color::Cyan)),
                Span::raw(&task.title),
            ]);
            ListItem::new(line)
        })
        .collect();

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::RIGHT)
        .title(title)
        .border_style(border_style);

    let highlight_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .bg(Color::DarkGray);

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style)
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, state);
}
