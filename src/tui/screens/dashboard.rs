use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    render_metrics(f, app, chunks[0]);
    render_activity(f, app, chunks[1]);
    render_run_info(f, app, chunks[2]);
    render_hotkeys(f, chunks[3]);
}

fn render_metrics(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let s = &app.stats;

    render_card(
        f, cols[0], "SIGNALS",
        &s.total_signals.to_string(),
        &format!("{} processed", s.processed_signals),
        Color::Cyan,
    );
    render_card(
        f, cols[1], "DRAFTS",
        &s.pending_drafts.to_string(),
        "pending",
        Color::Yellow,
    );
    render_card(
        f, cols[2], "PUBLISHED",
        &s.published_count.to_string(),
        "total",
        Color::Green,
    );

    let rate_str   = format!("{:.0}%", s.approval_rate * 100.0);
    let rate_color = if s.approval_rate >= 0.7 {
        Color::Green
    } else if s.approval_rate >= 0.4 {
        Color::Yellow
    } else {
        Color::Red
    };
    render_card(f, cols[3], "APPROVAL", &rate_str, "rate", rate_color);
}

fn render_card(
    f:        &mut Frame,
    area:     Rect,
    title:    &str,
    value:    &str,
    subtitle: &str,
    color:    Color,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(value)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center),
        layout[1],
    );
    f.render_widget(
        Paragraph::new(subtitle)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        layout[2],
    );
}

fn render_activity(f: &mut Frame, app: &App, area: Rect) {
    let s = &app.stats;

    let total_drafts =
        s.approved_drafts + s.pending_drafts + s.rejected_drafts;

    let data = vec![
        Bar::default()
            .value(s.total_signals as u64)
            .label("signals".into())
            .style(Style::default().fg(Color::Cyan)),
        Bar::default()
            .value(total_drafts as u64)
            .label("drafts".into())
            .style(Style::default().fg(Color::Yellow)),
        Bar::default()
            .value(s.approved_drafts as u64)
            .label("approved".into())
            .style(Style::default().fg(Color::Green)),
        Bar::default()
            .value(s.published_count as u64)
            .label("published".into())
            .style(Style::default().fg(Color::Magenta)),
    ];

    let chart = BarChart::default()
        .block(Block::default().borders(Borders::ALL).title(" Activity "))
        .data(BarGroup::default().bars(&data))
        .bar_width(10)
        .bar_gap(2)
        .value_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .label_style(Style::default().fg(Color::DarkGray));

    f.render_widget(chart, area);
}

fn render_run_info(f: &mut Frame, app: &App, area: Rect) {
    let last = app
        .stats
        .last_run
        .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "—".into());

    let next = app
        .stats
        .next_run
        .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "—".into());

    let text = Line::from(vec![
        Span::styled("Last run: ", Style::default().fg(Color::DarkGray)),
        Span::styled(last, Style::default().fg(Color::White)),
        Span::raw("   "),
        Span::styled("Next run: ", Style::default().fg(Color::DarkGray)),
        Span::styled(next, Style::default().fg(Color::White)),
    ]);

    f.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Schedule ")),
        area,
    );
}

fn render_hotkeys(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled("[r]", Style::default().fg(Color::Yellow)),
        Span::raw(" reload  "),
        Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
        Span::raw(" next screen  "),
        Span::styled("[1-5]", Style::default().fg(Color::Yellow)),
        Span::raw(" jump  "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ]);

    f.render_widget(
        Paragraph::new(line).alignment(Alignment::Center),
        area,
    );
}
