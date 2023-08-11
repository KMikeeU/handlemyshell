use std::{
    error::Error,
    io::{self, Stdout},
    sync::Arc,
    time::{Duration, Instant},
};

use app::App;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use session::Session;

use ui::*;

mod app;
mod session;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;

    let app = App::new();

    run(&mut terminal, app, Duration::from_millis(250))?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    Ok(terminal.show_cursor()?)
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut app: App,
    tick_rate: Duration,
) -> Result<(), Box<dyn Error>> {
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| draw_main(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        for _ in 0..app.network_bus.receiver.len() {
            let event = app
                .network_bus
                .receiver
                .try_recv()
                .expect("recv from channel failed, eventhough the length has been checked");

            match event {
                app::NetworkEvent::NewSession(s) => {
                    app.highest_session_number += 1;
                    let session = Session {
                        id: app.highest_session_number,
                        streams: s,
                    };

                    app.sessions.push(Arc::new(session))
                }
            }
        }

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('a') if key.modifiers == event::KeyModifiers::CONTROL => {
                            app.on_next();
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Tab => {
                            app.on_tab();
                        }
                        KeyCode::Down => {
                            app.on_down();
                        }
                        KeyCode::Up => {
                            app.on_up();
                        }
                        KeyCode::Char(' ') => {
                            app.on_space();
                        }
                        KeyCode::Enter => {
                            app.on_enter();
                        }
                        KeyCode::Char('c') => {
                            app.on_create();
                        }
                        KeyCode::Char('d') => {
                            app.on_delete();
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            // app.on_tick();
            last_tick = Instant::now();
        }
    }
}
