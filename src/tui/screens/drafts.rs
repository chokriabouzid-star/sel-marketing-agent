use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::tui::app::{App, ConfirmAction, EditMode};
use crate::models::{DraftStatus, Platform};

// ─── Platform helper — نستخدم Display trait الموجود ─────────────────────

fn platform_label(p: &Platform) -> String {
    p.to_string()
}

// ─── Quality helpers ──────────────────────────────────────────────────────

fn quality_color(score: Option<f32>) -> Color {
    match score {
        Some(s) if s >= 0.85 => Color::Green,
        Some(s) if s >= 0.70 => Color::Yellow,
        Some(s) if s >= 0.55 => Color::Rgb(255, 165, 0),
        Some(_)               => Color::Red,
        None                  => Color::DarkGray,
    }
}

fn quality_label(score: Option<f32>) -> &'static str {
    match score {
        Some(s) if s >= 0.85 => "Excellent",
        Some(s) if s >= 0.70 => "Good",
        Some(s) if s >= 0.55 => "Fair",
        Some(_)               => "Review",
        None                  => "—",
    }
}

fn score_str(score: Option<f32>) -> String {
    match score {
        Some(s) => format!("{s:.2}"),
        None    => "—".into(),
    }
}

// ─── Main render ──────────────────────────────────────────────────────────

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let pending: Vec<_> = app
        .drafts
        .iter()
        .filter(|d| d.status == DraftStatus::PendingApproval)
        .collect();

    if pending.is_empty() {
        render_empty(f, app, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(12),
            Constraint::Length(4),
            Constraint::Length(2),
        ])
        .split(area);

    let idx   = app.drafts_cursor.min(pending.len() - 1);
    let draft = pending[idx];

    render_card(f, app, draft, chunks[0]);
    render_session_summary(f, app, chunks[1]);
    render_progress(f, app, &pending, chunks[2]);

    if let Some(ConfirmAction::RejectDraft(_)) = &app.show_confirm {
        render_confirm_popup(f, f.size());
    }
}

fn render_card(
    f:     &mut Frame,
    app:   &App,
    draft: &crate::models::Draft,
    area:  Rect,
) {
    let border_color = quality_color(draft.quality_score);
    let title = format!(
        " Draft {}/{} | {} | Quality: {} ({}) ",
        app.drafts_cursor + 1,
        app.pending_drafts_count(),
        platform_label(&draft.platform),
        quality_label(draft.quality_score),
        score_str(draft.quality_score),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            title,
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // signal ref
            Constraint::Length(1),   // draft id
            Constraint::Min(4),      // content
            Constraint::Length(1),   // hotkeys
        ])
        .split(inner);

    // signal ref
    let signal_ref = draft.signal_id.as_deref().unwrap_or("—");
    let signal_url = draft.signal_url.as_deref().unwrap_or("");
    let signal_info = if signal_url.is_empty() {
        signal_ref.to_string()
    } else {
        format!("{signal_ref}  {signal_url}")
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Signal: ", Style::default().fg(Color::DarkGray)),
            Span::styled(signal_info, Style::default().fg(Color::White)),
        ]))
        .wrap(Wrap { trim: true }),
        inner_chunks[0],
    );

    // draft id
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&draft.id, Style::default().fg(Color::DarkGray)),
        ])),
        inner_chunks[1],
    );

    // content
    let content_text = if app.edit_mode == EditMode::Editing {
        format!("{}_", app.edit_buffer)
    } else {
        draft.content.clone()
    };

    let content_block = if app.edit_mode == EditMode::Editing {
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" Editing — [Enter] save  [Esc] cancel ")
    } else {
        Block::default().borders(Borders::NONE)
    };

    let content_style = if app.edit_mode == EditMode::Editing {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    f.render_widget(
        Paragraph::new(content_text)
            .block(content_block)
            .style(content_style)
            .wrap(Wrap { trim: false }),
        inner_chunks[2],
    );

    // hotkeys
    let hotkeys = if app.edit_mode == EditMode::Editing {
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green)),
            Span::raw(" save  "),
            Span::styled("[Esc]",   Style::default().fg(Color::Red)),
            Span::raw(" cancel"),
        ])
    } else {
        Line::from(vec![
            Span::styled("[a]",   Style::default().fg(Color::Green)),
            Span::raw(" Approve  "),
            Span::styled("[r]",   Style::default().fg(Color::Red)),
            Span::raw(" Reject  "),
            Span::styled("[e]",   Style::default().fg(Color::Yellow)),
            Span::raw(" Edit  "),
            Span::styled("[h/l]", Style::default().fg(Color::Cyan)),
            Span::raw(" Navigate"),
        ])
    };

    f.render_widget(
        Paragraph::new(hotkeys).alignment(Alignment::Center),
        inner_chunks[3],
    );
}

fn render_session_summary(f: &mut Frame, app: &App, area: Rect) {
    let approved = app.drafts.iter()
        .filter(|d| d.status == DraftStatus::Approved).count();
    let rejected = app.drafts.iter()
        .filter(|d| d.status == DraftStatus::Rejected).count();

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Session: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{approved} approved"),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{rejected} rejected"),
                Style::default().fg(Color::Red),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{} pending", app.pending_drafts_count()),
                Style::default().fg(Color::Yellow),
            ),
        ]))
        .block(Block::default().borders(Borders::TOP))
        .alignment(Alignment::Center),
        area,
    );
}

fn render_progress(
    f:       &mut Frame,
    app:     &App,
    pending: &[&crate::models::Draft],
    area:    Rect,
) {
    let dots: String = pending
        .iter()
        .enumerate()
        .map(|(i, _)| if i == app.drafts_cursor { "●" } else { "○" })
        .collect::<Vec<_>>()
        .join(" ");

    f.render_widget(
        Paragraph::new(dots)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center),
        area,
    );
}

fn render_empty(f: &mut Frame, app: &App, area: Rect) {
    let approved = app.drafts.iter()
        .filter(|d| d.status == DraftStatus::Approved).count();

    let msg = if approved > 0 {
        format!("All done! {approved} approved — run dispatch to publish")
    } else {
        "No pending drafts — run forge to generate content".into()
    };

    f.render_widget(
        Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Drafts ")),
        area,
    );
}

fn render_confirm_popup(f: &mut Frame, area: Rect) {
    let popup = crate::tui::ui::centered_rect(50, 25, area);
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Reject this draft?",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  [Y]", Style::default().fg(Color::Red)),
                Span::raw(" Yes  "),
                Span::styled("[N]",   Style::default().fg(Color::Green)),
                Span::raw(" Cancel"),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title(" Confirm "),
        ),
        popup,
    );
}
