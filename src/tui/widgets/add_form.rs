use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::domain::task::Queue;

/// Queues available for new tasks (not Done).
const ADD_QUEUES: [Queue; 4] = [Queue::Inbox, Queue::Now, Queue::Next, Queue::Later];

pub fn cycle_queue(current: Queue) -> Queue {
    let idx = ADD_QUEUES.iter().position(|q| *q == current).unwrap_or(0);
    ADD_QUEUES[(idx + 1) % ADD_QUEUES.len()]
}

pub fn render(frame: &mut Frame, title: &str, queue: Queue) {
    let area = centered_rect(50, 7, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Add Task ")
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // Title input
    let title_line = Line::from(vec![
        Span::styled("Title: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{title}\u{2588}"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(title_line), rows[0]);

    // Queue selector
    let queue_spans: Vec<Span> = ADD_QUEUES
        .iter()
        .map(|q| {
            if *q == queue {
                Span::styled(
                    format!("[{q}]"),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(format!(" {q} "), Style::default().fg(Color::DarkGray))
            }
        })
        .collect();

    let mut queue_line_spans = vec![Span::styled("Queue: ", Style::default().fg(Color::Yellow))];
    queue_line_spans.extend(queue_spans);
    frame.render_widget(Paragraph::new(Line::from(queue_line_spans)), rows[1]);

    // Spacer row[2]

    // Help
    let help = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(":create  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(":queue  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(":cancel"),
    ]);
    frame.render_widget(Paragraph::new(help), rows[3]);
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
