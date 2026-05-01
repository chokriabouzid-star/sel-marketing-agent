use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Frame,
};
use crate::tui::app::{App, Screen};
use super::screens;

pub fn render(f: &mut Frame, app: &App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    render_header(f, app, chunks[0]);
    render_screen(f, app, chunks[1]);
    render_status_bar(f, app, chunks[2]);

    if app.show_help {
        render_help(f, size);
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let tab_titles: Vec<Line> = [
        Screen::Dashboard,
        Screen::Signals,
        Screen::Drafts,
        Screen::Published,
        Screen::Settings,
    ]
    .iter()
    .enumerate()
    .map(|(i, screen)| {
        let label = format!(" {} {} ", i + 1, screen.label());
        if *screen == Screen::Drafts && app.stats.pending_drafts > 0 {
            Line::from(vec![
                Span::raw(label),
                Span::styled(
                    format!("[{}]", app.stats.pending_drafts),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Line::from(label)
        }
    })
    .collect();

    let selected = match app.current_screen {
        Screen::Dashboard => 0,
        Screen::Signals   => 1,
        Screen::Drafts    => 2,
        Screen::Published => 3,
        Screen::Settings  => 4,
    };

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    " SEL Marketing Agent ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(tabs, area);
}

fn render_screen(f: &mut Frame, app: &App, area: Rect) {
    match app.current_screen {
        Screen::Dashboard => screens::dashboard::render(f, app, area),
        Screen::Signals   => screens::signals::render(f, app, area),
        Screen::Drafts    => screens::drafts::render(f, app, area),
        Screen::Published => screens::published::render(f, app, area),
        Screen::Settings  => screens::settings::render(f, app, area),
    }
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let (msg, style) = match &app.status_message {
        Some(msg) => {
            let color = if app.status_is_error {
                Color::Red
            } else {
                Color::Green
            };
            (msg.as_str(), Style::default().fg(color))
        }
        None => (
            " [Tab] next screen  [?] help  [q] quit  [Ctrl+S] save",
            Style::default().fg(Color::DarkGray),
        ),
    };

    f.render_widget(
        Paragraph::new(msg).style(style),
        area,
    );
}

fn render_help(f: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 70, area);
    f.render_widget(Clear, popup);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Global",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        help_line("[q]",       "Quit"),
        help_line("[?]",       "Toggle help"),
        help_line("[Tab]",     "Next screen"),
        help_line("[1-5]",     "Jump to screen"),
        help_line("[Ctrl+S]",  "Save changes"),
        Line::from(""),
        Line::from(Span::styled(
            "  Signals",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        help_line("[j/k / ↑↓]", "Navigate"),
        help_line("[r]",         "Reload"),
        Line::from(""),
        Line::from(Span::styled(
            "  Drafts",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        help_line("[h/l / ←→]", "Previous / Next"),
        help_line("[a]",         "Approve"),
        help_line("[r]",         "Reject"),
        help_line("[e]",         "Edit inline"),
        help_line("[Enter]",     "Save edit"),
        help_line("[Esc]",       "Cancel edit"),
        Line::from(""),
        Line::from(Span::styled(
            "  Settings",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        help_line("[t]", "Test connections"),
        help_line("[c]", "Clear signals"),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "Press any key to close",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(
                    " Help ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
        ),
        popup,
    );
}

fn help_line(key: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{key:<14}"),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(desc, Style::default().fg(Color::White)),
    ])
}

// pub لأن drafts.rs يستدعيها
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
