mod config;
mod engine;
mod input;
mod tui;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use signal_hook::{consts::SIGTERM, flag};
use std::{io::stdout, time::Duration};

use crate::{
    config::session,
    input::handler,
    tui::{app::App, layout},
};

fn setup_terminal(
) -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut out = stdout();
    if let Err(e) = execute!(out, EnterAlternateScreen, Hide) {
        let _ = disable_raw_mode();
        return Err(e.into());
    }
    let backend = CrosstermBackend::new(out);
    match Terminal::new(backend) {
        Ok(terminal) => Ok(terminal),
        Err(e) => {
            let _ = disable_raw_mode();
            let _ = execute!(stdout(), LeaveAlternateScreen, Show);
            Err(e.into())
        }
    }
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) {
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, Show);
    let _ = terminal.show_cursor();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install panic hook to restore terminal before printing panic message
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, Show);
        original_hook(panic_info);
    }));

    // SIGTERM handler — sets flag atomically; main loop checks and saves before exit (AC3)
    let sigterm_received = Arc::new(AtomicBool::new(false));
    flag::register(SIGTERM, Arc::clone(&sigterm_received))?;

    let mut terminal = setup_terminal()?;
    let mut app = App::new();

    // Session restore (AC2, AC5, AC7)
    match session::load() {
        Ok(Some(state)) => {
            app.state = state;
        }
        Ok(None) => { /* no session file — start fresh, no message */ }
        Err(msg) => {
            app.error_message = Some(format!("Session file corrupted; starting fresh: {}", msg));
        }
    }

    loop {
        terminal.draw(|f| layout::render(f, &app))?;

        // SIGTERM check — save and exit cleanly (AC3)
        if sigterm_received.load(Ordering::Relaxed) {
            let _ = session::save(&app.state);
            break;
        }

        // 16ms timeout: ≤16ms keypress latency (within NFR2's 50ms budget), ~0% CPU when idle
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                let action = handler::handle_key(&app.mode, key);
                app.apply(action);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Save on clean quit (AC1)
    let _ = session::save(&app.state);

    restore_terminal(&mut terminal);
    Ok(())
}
