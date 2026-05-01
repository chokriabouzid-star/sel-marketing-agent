use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};
use crate::tui::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if app.published.is_empty() {
        f.render_widget(
            Paragraph::new("Nothing published yet — run dispatch after approving drafts")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL).title(" Published ")),
            area,
        );
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(area);

    render_table(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
}

fn render_table(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("#")
            .style(Style::default().fg(Color::DarkGray)),
        Cell::from("Content Preview")
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from("Platform")
            .style(Style::default().fg(Color::DarkGray)),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .published
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let preview: String = d.content.chars().take(50).collect();
            let preview = if d.content.len() > 50 {
                format!("{preview}…")
            } else {
                preview
            };

            let style = if i == app.published_cursor {
                Style::default().fg(Color::Black).bg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            // نستخدم Display trait المعرّف في models.rs
            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(preview),
                Cell::from(d.platform.to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Min(40),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Published ({}) ", app.published.len())),
    );

    let mut state = TableState::default();
    state.select(Some(app.published_cursor));
    f.render_stateful_widget(table, area, &mut state);
}

fn render_content(f: &mut Frame, app: &App, area: Rect) {
    let (title, content): (String, String) =
        match app.published.get(app.published_cursor) {
            None    => (" Content ".into(), "—".into()),
            Some(d) => {
                let t = " Full Content ".into();
                let signal_ref = d.signal_id.as_deref().unwrap_or("—");
                let signal_url = d.signal_url.as_deref().unwrap_or("—");
                let c = format!(
                    "{}\n\nPlatform: {}\nSignal ID:  {signal_ref}\nSignal URL: {signal_url}",
                    d.content,
                    d.platform,
                );
                (t, c)
            }
        };

    f.render_widget(
        Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false }),
        area,
    );
}
