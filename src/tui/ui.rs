use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::{
    app_state::{FocusedPanel, Mode, TuiApp},
    widgets,
};

pub fn draw(frame: &mut Frame, app: &mut TuiApp) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let main_area = outer[0];
    let status_area = outer[1];

    if is_triage_context(&app.mode) {
        draw_triage(frame, main_area, app);
    } else if app.mode == Mode::Search {
        draw_search(frame, main_area, app);
    } else {
        draw_normal(frame, main_area, app);
    }

    widgets::status_bar::render(frame, status_area, app);

    // Overlay: add form
    if app.mode == Mode::AddForm {
        widgets::add_form::render(frame, &app.add_title, app.add_queue);
    }
}

fn draw_normal(frame: &mut Frame, area: Rect, app: &mut TuiApp) {
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(14),
            Constraint::Min(20),
            Constraint::Percentage(35),
        ])
        .split(area);

    let sidebar_area = panels[0];
    let task_list_area = panels[1];
    let detail_area = panels[2];

    let focused = app.focused_panel;

    widgets::sidebar::render(frame, sidebar_area, app, focused == FocusedPanel::Sidebar);

    let filter = app.active_filter();
    let tasks: Vec<_> = match filter {
        super::app_state::QueueFilter::Single(queue) => {
            app.tasks.iter().filter(|t| t.queue == queue).collect()
        }
        super::app_state::QueueFilter::All => app.tasks.iter().collect(),
    };
    widgets::task_list::render(
        frame,
        task_list_area,
        filter,
        &tasks,
        &mut app.task_list_state,
        focused == FocusedPanel::TaskList,
    );

    let selected = app.selected_task().cloned();
    widgets::detail::render(
        frame,
        detail_area,
        selected.as_ref(),
        app.detail_scroll,
        focused == FocusedPanel::Detail,
    );
}

fn draw_triage(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let progress = format!("{}/{}", app.triage_index + 1, app.triage_task_ids.len());
    let task = app.current_triage_task();
    widgets::triage::render(frame, area, task, &progress);
}

fn draw_search(frame: &mut Frame, area: Rect, app: &mut TuiApp) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    // Search input
    let input = Paragraph::new(Line::from(vec![
        Span::styled("/ ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{}\u{2588}", app.search_query),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .title(format!(" Search ({} results) ", app.search_results.len())),
    );
    frame.render_widget(input, rows[0]);

    // Results list
    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .filter_map(|(task_id, queue)| {
            let task = app.tasks.iter().find(|t| t.id == *task_id)?;
            let line = Line::from(vec![
                Span::styled(
                    format!("[{:<5}] ", queue),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(format!("{:<8}", task.id), Style::default().fg(Color::Cyan)),
                Span::raw(&task.title),
            ]);
            Some(ListItem::new(line))
        })
        .collect();

    let highlight_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .bg(Color::DarkGray);

    let list = List::new(items)
        .highlight_style(highlight_style)
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, rows[1], &mut app.search_list_state);
}

fn is_triage_context(mode: &Mode) -> bool {
    matches!(
        mode,
        Mode::Triage
            | Mode::MoveTarget { from_triage: true }
            | Mode::ConfirmDelete {
                from_triage: true,
                ..
            }
    )
}
