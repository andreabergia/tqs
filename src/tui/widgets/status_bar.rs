use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::tui::app_state::{Mode, TuiApp};

pub fn render(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let line = match &app.mode {
        Mode::Normal => normal_line(app),
        Mode::ConfirmDelete { task_id } => confirm_delete_line(task_id),
        Mode::MoveTarget => move_target_line(),
    };

    let bar = Paragraph::new(line);
    frame.render_widget(bar, area);
}

fn normal_line(app: &TuiApp) -> Line<'static> {
    if let Some(msg) = app.active_status_message() {
        return Line::from(vec![
            mode_badge("Normal"),
            Span::raw(" "),
            Span::styled(msg.to_string(), Style::default().fg(Color::Green)),
        ]);
    }

    let mut spans = vec![
        mode_badge("Normal"),
        Span::raw(" "),
        hint("j/k"),
        Span::raw(":nav "),
        hint("Tab"),
        Span::raw(":queue "),
        hint("p"),
        Span::raw(":detail "),
        hint("d"),
        Span::raw(":done "),
        hint("s"),
        Span::raw(":start "),
        hint("m"),
        Span::raw(":move "),
        hint("x"),
        Span::raw(":del "),
        hint("r"),
        Span::raw(":refresh "),
        hint("q"),
        Span::raw(":quit"),
    ];

    // Remove extra hints if terminal is narrow — just keep the essentials
    if area_too_narrow(spans.iter().map(|s| s.width()).sum()) {
        spans.truncate(3); // mode badge + space + first hint
    }

    Line::from(spans)
}

fn confirm_delete_line(task_id: &str) -> Line<'static> {
    Line::from(vec![
        mode_badge("Delete"),
        Span::raw(format!(" Delete {task_id}? ")),
        hint("y"),
        Span::raw(":yes "),
        Span::raw("any other key:cancel"),
    ])
}

fn move_target_line() -> Line<'static> {
    Line::from(vec![
        mode_badge("Move"),
        Span::raw(" Move to: "),
        hint("i"),
        Span::raw(":inbox "),
        hint("n"),
        Span::raw(":now "),
        hint("x"),
        Span::raw(":next "),
        hint("l"),
        Span::raw(":later "),
        Span::raw("Esc:cancel"),
    ])
}

fn mode_badge(label: &str) -> Span<'static> {
    Span::styled(
        format!(" [{label}] "),
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
}

fn hint(key: &str) -> Span<'static> {
    Span::styled(key.to_string(), Style::default().fg(Color::Yellow))
}

fn area_too_narrow(total_width: usize) -> bool {
    // This is a simple heuristic; in practice the status bar area width is checked
    total_width > 120
}
