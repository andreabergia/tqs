use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::tui::app_state::{SidebarEntry, TuiApp};

pub fn render(frame: &mut Frame, area: Rect, app: &TuiApp, focused: bool) {
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
                let count = app.queue_count(*queue);
                let is_active = i == app.active_sidebar_index;
                queue_item(&queue.to_string(), count, is_active)
            }
            SidebarEntry::All => {
                let count = app.total_count();
                let is_active = i == app.active_sidebar_index;
                queue_item("all", count, is_active)
            }
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
