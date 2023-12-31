use std::io::Stdout;

use crate::app::{App, LocalTabs, TerminalSize};

use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::{Color, Style},
    text::{self},
    widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table, Tabs},
    Frame,
};
use tui_term::widget::PseudoTerminal;

pub fn draw_listeners(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App, area: Rect) {
    let border_color = match app.active_tab {
        LocalTabs::Listeners => Color::Green,
        _ => Color::DarkGray,
    };

    let widget = Table::new(app.listeners.iter().enumerate().map(|(id, l)| {
        let mut row = Row::new([
            Cell::from(id.to_string()),
            Cell::from(l.port.to_string()),
            Cell::from(l.status.to_string()),
        ]);

        if app.active_tab == LocalTabs::Listeners && id == app.listener_selection_index {
            row = row.style(Style::default().bg(Color::Green));
        }

        row
    }))
    .header(
        Row::new([Cell::from("id"), Cell::from("Port"), Cell::from("Status")])
            .style(Style::default().fg(Color::DarkGray)),
    )
    .widths(
        [
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Length(10),
        ]
        .as_ref(),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Listeners")
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .padding(Padding::new(1, 1, 0, 0)),
    );

    f.render_widget(widget, area);
}

pub fn draw_sessions(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App, area: Rect) {
    let border_color = match app.active_tab {
        LocalTabs::Sessions => Color::Green,
        _ => Color::DarkGray,
    };

    let rows = {
        let result: Vec<Row> = app
            .sessions
            .iter()
            .enumerate()
            .map(|(id, s)| {
                // If this row is selected, make it's text yellow and add a * to the left of it
                let is_selected = match &app.session {
                    Some(selected_session) => s.id == selected_session.id,
                    None => false,
                };

                let row = if is_selected {
                    Row::new([Cell::from("*"), Cell::from(s.id.to_string())])
                        .style(Style::default().fg(Color::Yellow))
                } else {
                    Row::new([Cell::from(id.to_string()), Cell::from(s.id.to_string())])
                };


                if app.active_tab == LocalTabs::Sessions && id == app.session_selection_index {
                    row.style(Style::default().bg(Color::Green))
                } else {
                    row
                }
            })
            .collect();

        result
    };

    let widget = Table::new(rows)
        .header(
            Row::new([Cell::from("i"), Cell::from("id")])
            .style(Style::default().fg(Color::DarkGray))
        )
        .widths(
            [
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
            ]
            .as_ref(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sessions")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .padding(Padding::new(1, 1, 0, 0)),
        );

    f.render_widget(widget, area);
}

pub fn draw_local(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);

    draw_listeners(f, app, chunks[0]);
    draw_sessions(f, app, chunks[1]);
}

pub fn draw_remote(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App, area: Rect) {
    match app.session.as_mut() {
        Some(session) => {
            app.remote_size = TerminalSize::new(area.y, area.x);

            {
                let term = session.term.lock().unwrap();
    
                // TODO: Fix this
                // if term.screen().size().0 != area.x && term.screen().size().1 != area.y {
                //     term.set_size(area.x, area.y);
                // }
                
    
                let screen = term.screen();
                let pseudo_term = PseudoTerminal::new(screen);
    
                f.render_widget(pseudo_term, area);
            }

        }
        None => {
            let paragraph = Paragraph::new("You aren't connected to a session yet. Switch to the local tab and select a session by pressing space.");
            f.render_widget(paragraph, area);
        }
    }
}

pub fn draw_main(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let titles = app
        .tabs
        .titles
        .iter()
        .map(|s| text::Line::from(*s))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(Style::default().fg(Color::Green))
        .select(app.tabs.index);

    f.render_widget(tabs, chunks[0]);

    match app.tabs.index {
        0 => {
            draw_local(f, app, chunks[1]);
        }
        1 => {
            draw_remote(f, app, chunks[1]);
        }
        _ => {}
    }
}
