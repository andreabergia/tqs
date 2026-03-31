use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::domain::task::Task;

pub fn render(frame: &mut Frame, area: Rect, task: Option<&Task>, progress: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Triage {progress} "))
        .border_style(Style::default().fg(Color::Yellow));

    let Some(task) = task else {
        let empty = Paragraph::new("No more tasks").block(block);
        frame.render_widget(empty, area);
        return;
    };

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(inner);

    // Header: task id + title
    let header = Line::from(vec![
        Span::styled(format!("{}  ", task.id), Style::default().fg(Color::Cyan)),
        Span::styled(&task.title, Style::default().add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(Paragraph::new(header), rows[0]);

    // Body preview
    let body_lines: Vec<Line> = task
        .body
        .lines()
        .map(|l| Line::from(l.to_string()))
        .collect();
    let body = Paragraph::new(body_lines).wrap(Wrap { trim: false });
    frame.render_widget(body, rows[1]);

    // Action hints
    let hints = Line::from(vec![
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(":now "),
        Span::styled("x", Style::default().fg(Color::Yellow)),
        Span::raw(":next "),
        Span::styled("l", Style::default().fg(Color::Yellow)),
        Span::raw(":later "),
        Span::styled("d", Style::default().fg(Color::Yellow)),
        Span::raw(":done "),
        Span::styled("D", Style::default().fg(Color::Yellow)),
        Span::raw(":delete "),
        Span::styled("e", Style::default().fg(Color::Yellow)),
        Span::raw(":edit "),
        Span::styled("s", Style::default().fg(Color::Yellow)),
        Span::raw(":skip "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(":quit"),
    ]);
    frame.render_widget(Paragraph::new(hints), rows[2]);
}
