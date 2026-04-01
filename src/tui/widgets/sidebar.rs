use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use super::panel_border_style;
use crate::tui::app_state::{SidebarEntry, TuiApp};

pub fn render(frame: &mut Frame, area: Rect, app: &TuiApp, focused: bool) {
    let counts = app.queue_counts();
    let items: Vec<ListItem> = app
        .sidebar_entries()
        .iter()
        .enumerate()
        .map(|(i, entry)| match entry {
            SidebarEntry::Separator => ListItem::new(Line::from(Span::styled(
                "  ──────────",
                Style::default().fg(Color::DarkGray),
            ))),
            SidebarEntry::Queue(queue) => {
                let is_active = i == app.active_sidebar_index;
                queue_item(&queue.to_string(), counts.get(*queue), is_active)
            }
            SidebarEntry::All => {
                let is_active = i == app.active_sidebar_index;
                queue_item("all", counts.total, is_active)
            }
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Queues ")
        .border_style(panel_border_style(focused));
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn queue_item(label: &str, count: usize, is_active: bool) -> ListItem<'static> {
    let marker = if is_active { ">" } else { " " };
    let line = Line::from(vec![
        Span::raw(format!("{marker} ")),
        Span::styled(
            format!("{:<6}", label),
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
}
