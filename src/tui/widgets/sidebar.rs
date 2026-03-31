use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::tui::app_state::TuiApp;

pub fn render(frame: &mut Frame, area: Rect, app: &TuiApp, focused: bool) {
    let items: Vec<ListItem> = app
        .sidebar_queues()
        .iter()
        .enumerate()
        .map(|(i, queue)| {
            let count = app.queue_count(*queue);
            let is_active = i == app.active_queue_index;

            let marker = if is_active { ">" } else { " " };
            let line = Line::from(vec![
                Span::raw(format!("{marker} ")),
                Span::styled(
                    format!("{:<6}", queue.to_string()),
                    if is_active {
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Magenta)
                    },
                ),
                Span::styled(format!("{count:>3}"), Style::default().fg(Color::Yellow)),
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
        .borders(Borders::ALL)
        .title(" Queues ")
        .border_style(border_style);
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
