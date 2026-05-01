use anyhow::Result;
use crossterm::{
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};
use sel_marketing_agent::tui::{app::App, events, ui};
use std::{io, time::Duration};

fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let data_dir = std::env::var("DATA_DIR")
        .unwrap_or_else(|_| "data".into());

    // ─── setup terminal ───────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend  = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    // ─── init app ─────────────────────────────────────────────────────────
    let mut app = App::new(data_dir);
    app.load_data().unwrap_or_else(|e| {
        app.set_error(format!("Load error: {e}"));
    });

    // ─── event loop ───────────────────────────────────────────────────────
    loop {
        term.draw(|f| ui::render(f, &app))?;

        if let Some(event) = events::poll_event(Duration::from_millis(50))? {
            use crossterm::event::Event;
            if let Event::Key(key) = event {
                events::handle_key(&mut app, key);
            }
        }

        if app.should_quit {
            app.save_data().ok();
            break;
        }
    }

    // ─── restore terminal ─────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;

    println!("SEL Marketing Agent — goodbye!");
    Ok(())
}
