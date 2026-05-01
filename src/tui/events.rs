use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use super::app::{App, ConfirmAction, ConnectionStatus, EditMode, Screen};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // ─── Edit mode ────────────────────────────────────────────────────────
    if app.edit_mode == EditMode::Editing {
        match key.code {
            KeyCode::Esc   => app.cancel_edit(),
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    app.edit_buffer.push('\n');
                } else {
                    app.save_edit();
                }
            }
            KeyCode::Backspace => { app.edit_buffer.pop(); }
            KeyCode::Char(c)   => app.edit_buffer.push(c),
            _ => {}
        }
        return;
    }

    // ─── Confirm popup ────────────────────────────────────────────────────
    if let Some(confirm) = app.show_confirm.clone() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                match confirm {
                    ConfirmAction::RejectDraft(id) => app.confirm_reject(&id),
                    ConfirmAction::ClearSignals    => {
                        app.signals.clear();
                        app.show_confirm = None;
                        app.recalculate_stats();
                        app.set_status("Signals cleared".into());
                    }
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.show_confirm = None;
            }
            _ => {}
        }
        return;
    }

    // ─── Help popup ───────────────────────────────────────────────────────
    if app.show_help {
        app.show_help = false;
        return;
    }

    // ─── Global keys ──────────────────────────────────────────────────────
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        KeyCode::Char('?')  => app.show_help = true,
        KeyCode::Tab        => app.next_screen(),
        KeyCode::BackTab    => app.prev_screen(),
        KeyCode::Char('1')  => app.current_screen = Screen::Dashboard,
        KeyCode::Char('2')  => app.current_screen = Screen::Signals,
        KeyCode::Char('3')  => app.current_screen = Screen::Drafts,
        KeyCode::Char('4')  => app.current_screen = Screen::Published,
        KeyCode::Char('5')  => app.current_screen = Screen::Settings,
        KeyCode::Char('s')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            match app.save_data() {
                Ok(_)  => {}
                Err(e) => app.set_error(format!("Save error: {e}")),
            }
        }
        _ => handle_screen_key(app, key),
    }
}

fn handle_screen_key(app: &mut App, key: KeyEvent) {
    match app.current_screen {
        Screen::Dashboard => handle_dashboard(app, key),
        Screen::Signals   => handle_signals(app, key),
        Screen::Drafts    => handle_drafts(app, key),
        Screen::Published => handle_published(app, key),
        Screen::Settings  => handle_settings(app, key),
    }
}

fn handle_dashboard(app: &mut App, key: KeyEvent) {
    if let KeyCode::Char('r') = key.code {
        match app.load_data() {
            Ok(_)  => app.set_status("Reloaded".into()),
            Err(e) => app.set_error(format!("{e}")),
        }
    }
}

fn handle_signals(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up   | KeyCode::Char('k') => app.cursor_up(),
        KeyCode::Down | KeyCode::Char('j') => app.cursor_down(),
        KeyCode::Char('r') => {
            match app.load_data() {
                Ok(_)  => app.set_status("Reloaded".into()),
                Err(e) => app.set_error(format!("{e}")),
            }
        }
        _ => {}
    }
}

fn handle_drafts(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Right | KeyCode::Char('l') => app.next_draft(),
        KeyCode::Left  | KeyCode::Char('h') => app.prev_draft(),
        KeyCode::Char('a') => app.approve_current_draft(),
        KeyCode::Char('r') => app.reject_current_draft(),
        KeyCode::Char('e') => app.enter_edit_mode(),
        _ => {}
    }
}

fn handle_published(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up   | KeyCode::Char('k') => app.cursor_up(),
        KeyCode::Down | KeyCode::Char('j') => app.cursor_down(),
        _ => {}
    }
}

fn handle_settings(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('c') => {
            app.show_confirm = Some(ConfirmAction::ClearSignals);
        }
        KeyCode::Char('t') => {
            // اختبار فعلي للـ APIs
            test_connections(app);
        }
        _ => {}
    }
}

/// اختبار الاتصالات — sync باستخدام tokio runtime
fn test_connections(app: &mut App) {
    app.set_status("Testing connections...".into());

    let groq_key     = std::env::var("GROQ_API_KEY").unwrap_or_default();
    let github_token = std::env::var("GITHUB_TOKEN").unwrap_or_default();
    let tg_token     = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let devto_key    = std::env::var("DEVTO_API_KEY").unwrap_or_default();

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r)  => r,
        Err(e) => {
            app.set_error(format!("Runtime error: {e}"));
            return;
        }
    };

    // ─── Groq ─────────────────────────────────────────────────────────────
    app.api_status.groq = if groq_key.is_empty() {
        ConnectionStatus::NoKey
    } else {
        let key = groq_key.clone();
        match rt.block_on(test_groq(&key)) {
            Ok(model) => ConnectionStatus::Connected(model),
            Err(e)    => ConnectionStatus::Disabled(e.to_string()),
        }
    };

    // ─── GitHub ───────────────────────────────────────────────────────────
    app.api_status.github = if github_token.is_empty() {
        ConnectionStatus::NoKey
    } else {
        let tok = github_token.clone();
        match rt.block_on(test_github(&tok)) {
            Ok(user) => ConnectionStatus::Connected(user),
            Err(e)   => ConnectionStatus::Disabled(e.to_string()),
        }
    };

    // ─── Telegram ─────────────────────────────────────────────────────────
    app.api_status.telegram = if tg_token.is_empty() {
        ConnectionStatus::NoKey
    } else {
        let tok = tg_token.clone();
        match rt.block_on(test_telegram(&tok)) {
            Ok(name) => ConnectionStatus::Connected(name),
            Err(e)   => ConnectionStatus::Disabled(e.to_string()),
        }
    };

    // ─── Dev.to ───────────────────────────────────────────────────────────
    app.api_status.devto = if devto_key.is_empty() {
        ConnectionStatus::NoKey
    } else {
        ConnectionStatus::Connected("key present".into())
    };

    app.set_status("Connections tested".into());
}

async fn test_groq(api_key: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let resp   = client
        .get("https://api.groq.com/openai/v1/models")
        .header("Authorization", format!("Bearer {api_key}"))
        .timeout(Duration::from_secs(5))
        .send()
        .await?;

    if resp.status().is_success() {
        Ok("llama-3.3-70b-versatile".into())
    } else {
        anyhow::bail!("HTTP {}", resp.status())
    }
}

async fn test_github(token: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let resp   = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("token {token}"))
        .header("User-Agent", "sel-marketing-agent")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;

    if resp.status().is_success() {
        let json: serde_json::Value = resp.json().await?;
        let login = json["login"]
            .as_str()
            .unwrap_or("connected")
            .to_string();
        Ok(login)
    } else {
        anyhow::bail!("HTTP {}", resp.status())
    }
}

async fn test_telegram(token: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let url    = format!(
        "https://api.telegram.org/bot{token}/getMe"
    );
    let resp   = client
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await?;

    if resp.status().is_success() {
        let json: serde_json::Value = resp.json().await?;
        let name = json["result"]["username"]
            .as_str()
            .unwrap_or("connected")
            .to_string();
        Ok(format!("@{name}"))
    } else {
        anyhow::bail!("HTTP {}", resp.status())
    }
}

pub fn poll_event(timeout: Duration) -> anyhow::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
