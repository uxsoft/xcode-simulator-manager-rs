use humansize::{DECIMAL, format_size};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap};

use crate::app::{App, Modal};
use crate::simctl::DeviceState;

const ACCENT: Color = Color::Cyan;
const DANGER: Color = Color::Red;
const DIM: Color = Color::DarkGray;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    draw_header(frame, layout[0], app);
    draw_table(frame, layout[1], app);
    draw_footer(frame, layout[2], app);

    match &app.modal {
        Modal::None => {}
        Modal::Confirm => draw_confirm_modal(frame, app),
        Modal::Deleting => draw_centered_modal(
            frame,
            "Deleting…",
            "Shutting down booted simulators and removing devices.",
            ACCENT,
        ),
        Modal::Error(msg) => draw_centered_modal(frame, "Error", msg, DANGER),
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let total_count = app.sims.len();
    let total = format_size(app.total_bytes(), DECIMAL);
    let selected_count = app.selected.len();
    let selected = format_size(app.selected_bytes(), DECIMAL);
    let scanning = if app.scanning { " · scanning…" } else { "" };

    let line = Line::from(vec![
        Span::styled(
            " Xcode Simulators ",
            Style::default().fg(Color::Black).bg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  {total_count} devices · {total} total")),
        Span::styled(
            format!("   {selected_count} selected · {selected}"),
            Style::default().fg(if selected_count > 0 { DANGER } else { DIM }),
        ),
        Span::styled(scanning.to_string(), Style::default().fg(DIM)),
        Span::raw("   sort: "),
        Span::styled(app.sort.label(), Style::default().fg(ACCENT)),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_table(frame: &mut Frame, area: Rect, app: &mut App) {
    let header = Row::new(vec![
        Cell::from(""),
        Cell::from("Name"),
        Cell::from("Runtime"),
        Cell::from("State"),
        Cell::from("Size"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD).fg(ACCENT));

    let rows: Vec<Row> = app
        .sims
        .iter()
        .map(|sim| {
            let is_selected = app.selected.contains(&sim.udid);
            let mark = if is_selected { "✓" } else { " " };
            let mark_style = if is_selected {
                Style::default().fg(DANGER).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let mut name = Span::raw(sim.name.clone());
            if !sim.is_available {
                name = name.style(Style::default().fg(DIM));
            }

            let state_label = match sim.state {
                DeviceState::Booted => "● Booted",
                DeviceState::Shutdown if !sim.is_available => "Unavailable",
                DeviceState::Shutdown => "Shutdown",
                DeviceState::Other => sim.state.label(),
            };
            let state_style = match sim.state {
                DeviceState::Booted => Style::default().fg(Color::Green),
                _ if !sim.is_available => Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
                _ => Style::default(),
            };

            let size_text = match sim.size_bytes {
                Some(b) => format_size(b, DECIMAL),
                None => "…".to_string(),
            };
            let size_style = if sim.size_bytes.is_none() {
                Style::default().fg(DIM)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(Span::styled(mark, mark_style)),
                Cell::from(name),
                Cell::from(sim.runtime.clone()),
                Cell::from(Span::styled(state_label, state_style)),
                Cell::from(Span::styled(size_text, size_style)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Min(20),
            Constraint::Length(14),
            Constraint::Length(13),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .row_highlight_style(Style::default().bg(Color::Rgb(40, 40, 60)).add_modifier(Modifier::BOLD))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));

    frame.render_stateful_widget(table, area, &mut app.table);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let hints = if matches!(app.modal, Modal::Confirm) {
        " y confirm · n / esc cancel "
    } else if matches!(app.modal, Modal::Error(_)) {
        " any key dismiss "
    } else {
        " ↑/↓ move · space select · s sort · r refresh · d delete · q quit "
    };
    let footer = Paragraph::new(Line::from(Span::styled(
        hints,
        Style::default().fg(DIM),
    )));
    frame.render_widget(footer, area);
}

fn draw_confirm_modal(frame: &mut Frame, app: &App) {
    let sims = app.selected_sims();
    let bytes = format_size(app.selected_bytes(), DECIMAL);
    let count = sims.len();

    let mut lines = vec![
        Line::from(Span::styled(
            format!("Delete {count} simulator{}?", if count == 1 { "" } else { "s" }),
            Style::default().fg(DANGER).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("Reclaiming {bytes}")),
        Line::from(""),
    ];
    let max_show = 8usize;
    for sim in sims.iter().take(max_show) {
        let booted = if sim.state == DeviceState::Booted {
            " (will shut down)"
        } else {
            ""
        };
        lines.push(Line::from(format!("  • {} — {}{}", sim.name, sim.runtime, booted)));
    }
    if count > max_show {
        lines.push(Line::from(format!("  …and {} more", count - max_show)));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press y to delete · n / esc to cancel",
        Style::default().fg(DIM),
    )));

    let area = centered_rect(60, 60, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DANGER))
        .title(" Confirm deletion ");
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_centered_modal(frame: &mut Frame, title: &str, body: &str, color: Color) {
    let area = centered_rect(50, 30, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(format!(" {title} "));
    let para = Paragraph::new(body.to_string())
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(v[1])[1]
}
