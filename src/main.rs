use std::{
    error::Error,
    io::{self, Stdout},
    sync::{Arc, Mutex},
    time::{Duration, Instant}, thread,
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
use vt100::Parser;

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
                    s.s2c.sender.send(b"script -qc /usr/bin/bash /dev/null 2>&1\n".to_vec()).unwrap();
                    s.s2c.sender.send(b"stty -echo nl\n".to_vec()).unwrap();
                    // s.s2c.sender.send(b"stty -echo nl lnext ^V; export PS1='$>';\n".to_vec()).unwrap();

                    app.highest_session_number += 1;
                    let session = Session {
                        id: app.highest_session_number,
                        bus: s,
                        // TODO: make these numbers adjust to the terminal size
                        term: Mutex::new(Parser::new(20, 80, 0)),
                    };

                    let session = Arc::new(session);

                    // Receive from channel and update PTY
                    {
                        let rx = session.bus.c2s.receiver.clone();
                        let session = session.clone();

                        thread::spawn(move || {
                            while let Ok(buf) = rx.recv() {
                                session.term.lock().unwrap().process(&buf);
                            }
                        });
                    }


                    app.sessions.push(session);
                }
            }
        }

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // In all cases, CTRL + a switches between local and remote
                    if key.code == KeyCode::Char('a') && key.modifiers == event::KeyModifiers::CONTROL {
                        app.on_next();
                    } else if app.tabs.index == 1 {
                        // We're on the remote, pass through the events
                        app.on_remote_interact(key);
                    } else {
                        match key.code {
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
        }

        if last_tick.elapsed() >= tick_rate {
            // app.on_tick();
            last_tick = Instant::now();
        }
    }
}
