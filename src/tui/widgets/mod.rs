pub mod add_form;
pub mod detail;
pub mod sidebar;
pub mod status_bar;
pub mod task_list;
pub mod triage;

use ratatui::style::{Color, Style};

pub fn panel_border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}
