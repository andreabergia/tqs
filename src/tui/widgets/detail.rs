use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::panel_border_style;
use crate::domain::task::Task;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    task: Option<&Task>,
    scroll_offset: u16,
    focused: bool,
) {
    let border_style = panel_border_style(focused);

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
