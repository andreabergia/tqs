use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use super::{app_state::TuiApp, widgets};

pub fn draw(frame: &mut Frame, app: &mut TuiApp) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let main_area = outer[0];
    let status_area = outer[1];

    let panel_constraints = if app.detail_visible {
        vec![
            Constraint::Length(14),
            Constraint::Min(20),
            Constraint::Percentage(35),
        ]
    } else {
        vec![Constraint::Length(14), Constraint::Min(20)]
    };

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(panel_constraints)
        .split(main_area);

    let sidebar_area = panels[0];
    let task_list_area = panels[1];

    widgets::sidebar::render(frame, sidebar_area, app);

    let queue = app.active_queue();
    let tasks: Vec<_> = app.tasks.iter().filter(|t| t.queue == queue).collect();
    widgets::task_list::render(
        frame,
        task_list_area,
        queue,
        &tasks,
        &mut app.task_list_state,
    );

    if app.detail_visible && panels.len() > 2 {
        let detail_area = panels[2];
        let selected = app.selected_task().cloned();
        widgets::detail::render(frame, detail_area, selected.as_ref(), app.detail_scroll);
    }

    widgets::status_bar::render(frame, status_area, app);
}
