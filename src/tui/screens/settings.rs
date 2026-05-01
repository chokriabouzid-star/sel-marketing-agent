use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::{App, ConnectionStatus};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(9),
            Constraint::Length(7),
            Constraint::Min(0),
        ])
        .split(area);

    render_api_status(f, app, chunks[0]);
    render_data_files(f, app, chunks[1]);
    render_hotkeys(f, chunks[2]);
}

fn render_api_status(f: &mut Frame, app: &App, area: Rect) {
    let rows = vec![
        ("Groq API",     &app.api_status.groq),
        ("GitHub",       &app.api_status.github),
        ("Telegram Bot", &app.api_status.telegram),
        ("Reddit",       &app.api_status.reddit),
        ("Dev.to",       &app.api_status.devto),
    ];

    let mut lines: Vec<Line> = vec![Line::from("")];
    for (name, status) in rows {
        let (icon, detail, color) = match status {
            ConnectionStatus::Connected(info)  =>
                ("✅", format!("Connected   {info}"), Color::Green),
            ConnectionStatus::Disabled(reason) =>
                ("--", format!("Disabled    ({reason})"), Color::DarkGray),
            ConnectionStatus::NoKey            =>
                ("[!]", "No key (optional)".into(), Color::Yellow),
            ConnectionStatus::Unknown          =>
                ("[?]", "Not tested — press [t]".into(), Color::DarkGray),
        };

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{icon}  "),
                Style::default().fg(color),
            ),
            Span::styled(
                format!("{name:<15}"),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(detail, Style::default().fg(color)),
        ]));
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" API Status ")),
        area,
    );
}

fn render_data_files(f: &mut Frame, app: &App, area: Rect) {
    let s = &app.stats;
    let total_drafts = s.pending_drafts + s.approved_drafts + s.rejected_drafts;

    let lines = vec![
        Line::from(""),
        file_line("signals.jsonl",   s.total_signals),
        file_line("drafts.jsonl",    total_drafts),
        file_line("published.jsonl", s.published_count),
    ];

    f.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Data Files ")),
        area,
    );
}

fn file_line(name: &'static str, count: usize) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{name:<22}"),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            format!("{count} entries"),
            Style::default().fg(Color::White),
        ),
    ])
}

fn render_hotkeys(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(vec![
            Span::styled("[t]",      Style::default().fg(Color::Yellow)),
            Span::raw(" Test connections  "),
            Span::styled("[c]",      Style::default().fg(Color::Red)),
            Span::raw(" Clear signals  "),
            Span::styled("[Ctrl+S]", Style::default().fg(Color::Green)),
            Span::raw(" Save"),
        ]),
    ];
    f.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::TOP)),
        area,
    );
}
