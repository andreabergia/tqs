use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::tui::app_state::Mode;

pub fn render(frame: &mut Frame, area: Rect, mode: Mode) {
    let mode_label = match mode {
        Mode::Normal => "Normal",
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" [{mode_label}] "),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw(":nav  "),
        Span::styled("Tab/h/l", Style::default().fg(Color::Yellow)),
        Span::raw(":queue  "),
        Span::styled("1-5", Style::default().fg(Color::Yellow)),
        Span::raw(":jump  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(":quit"),
    ]);

    let bar = Paragraph::new(line);
    frame.render_widget(bar, area);
}
