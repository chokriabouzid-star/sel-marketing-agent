use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};
use crate::tui::app::App;
use crate::models::SignalSource;

fn source_str(s: &SignalSource) -> String {
    s.to_string()
}

fn source_short(s: &SignalSource) -> &'static str {
    match s {
        SignalSource::RedditLocalllama      => "r/LocalLLaMA",
        SignalSource::RedditProgramming     => "r/programming",
        SignalSource::RedditMachinelearning => "r/ML",
        SignalSource::RedditRust            => "r/rust",
        SignalSource::HackerNews            => "HN",
        SignalSource::GithubCompetitor(_)   => "GitHub",
    }
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    render_table(f, app, chunks[0]);
    render_preview(f, app, chunks[1]);
}

fn render_table(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("#")
            .style(Style::default().fg(Color::DarkGray)),
        Cell::from("Title")
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Cell::from("Source")
            .style(Style::default().fg(Color::DarkGray)),
        Cell::from("Score")
            .style(Style::default().fg(Color::DarkGray)),
        Cell::from("Done")
            .style(Style::default().fg(Color::DarkGray)),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .signals
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let icon      = if s.processed { "✅" } else { "⏳" };
            let score_str = s.score
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".into());
            let title_short: String = s.title.chars().take(44).collect();
            let title_display = if s.title.len() > 44 {
                format!("{title_short}…")
            } else {
                title_short
            };

            let row_style = if i == app.signals_cursor {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else if s.processed {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(title_display),
                Cell::from(source_short(&s.source)),
                Cell::from(score_str),
                Cell::from(icon),
            ])
            .style(row_style)
        })
        .collect();

    let unprocessed = app.signals.iter().filter(|s| !s.processed).count();
    let title = format!(
        " Signals ({} total — {unprocessed} unprocessed) ",
        app.signals.len()
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Min(40),
            Constraint::Length(14),
            Constraint::Length(7),
            Constraint::Length(5),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut state = TableState::default();
    state.select(Some(app.signals_cursor));
    f.render_stateful_widget(table, area, &mut state);
}

fn render_preview(f: &mut Frame, app: &App, area: Rect) {
    let (title, content) = match app.signals.get(app.signals_cursor) {
        None    => (" Preview ".to_string(), "—".to_string()),
        Some(s) => {
            let max = s.title.len().min(35);
            let t = format!(" Preview — {} ", &s.title[..max]);
            let c = format!(
                "{}\n\nSource: {}\nScore: {} | Comments: {}\nURL: {}",
                s.title,
                source_str(&s.source),
                s.score.map(|n| n.to_string()).unwrap_or_else(|| "—".into()),
                s.comment_count.map(|n| n.to_string()).unwrap_or_else(|| "—".into()),
                s.url.as_deref().unwrap_or("—"),
            );
            (t, c)
        }
    };

    f.render_widget(
        Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true }),
        area,
    );
}
