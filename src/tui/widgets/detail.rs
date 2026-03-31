use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
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
            .borders(Borders::NONE)
            .title(" Detail ")
            .border_style(border_style);
        let empty = Paragraph::new("No task selected").block(block);
        frame.render_widget(empty, area);
        return;
    };

    let title = format!(" {} [{}] ", task.id, task.queue);
    let block = Block::default()
        .borders(Borders::NONE)
        .title(title)
        .border_style(border_style);

    let mut lines = vec![
        Line::from(Span::styled(
            &task.title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for text_line in task.body.lines() {
        lines.push(Line::from(text_line.to_string()));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset, 0));

    frame.render_widget(paragraph, area);
}
