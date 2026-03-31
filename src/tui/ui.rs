use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use super::{
    app_state::{Mode, TuiApp},
    widgets,
};

pub fn draw(frame: &mut Frame, app: &mut TuiApp) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let main_area = outer[0];
    let status_area = outer[1];

    if app.mode == Mode::Triage {
        draw_triage(frame, main_area, app);
    } else {
        draw_normal(frame, main_area, app);
    }

    widgets::status_bar::render(frame, status_area, app);

    // Overlay: add form
    if app.mode == Mode::AddForm {
        widgets::add_form::render(frame, &app.add_title, app.add_queue);
    }
}

fn draw_normal(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut TuiApp) {
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
        .split(area);

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
}

fn draw_triage(frame: &mut Frame, area: ratatui::layout::Rect, app: &TuiApp) {
    let progress = format!("{}/{}", app.triage_index + 1, app.triage_task_ids.len());
    let task = app.current_triage_task();
    widgets::triage::render(frame, area, task, &progress);
}
