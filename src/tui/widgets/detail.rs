use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::domain::task::Task;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    task: Option<&Task>,
    scroll_offset: u16,
    focused: bool,
) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let Some(task) = task else {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Task detail ")
            .border_style(border_style);
        let empty = Paragraph::new("No task selected").block(block);
        frame.render_widget(empty, area);
        return;
    };

    let title = format!(" Task detail: {} ", task.title);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    let lines: Vec<Line> = task
        .body
        .lines()
        .map(|l| Line::from(l.to_string()))
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset, 0));

    frame.render_widget(paragraph, area);
}
