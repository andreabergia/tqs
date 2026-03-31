use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::domain::task::Task;

pub fn render(frame: &mut Frame, area: Rect, task: Option<&Task>, scroll_offset: u16) {
    let Some(task) = task else {
        let block = Block::default().borders(Borders::LEFT).title(" Detail ");
        let empty = Paragraph::new("No task selected").block(block);
        frame.render_widget(empty, area);
        return;
    };

    let title = format!(" {} [{}] ", task.id, task.queue);
    let block = Block::default().borders(Borders::LEFT).title(title);

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
