use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
    thread,
};

use crossbeam::channel::{unbounded, Receiver, Sender};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use strum_macros::Display;

use crate::session::{listen, Session, SessionBus};

pub struct App<'a> {
    pub tabs: TabsState<'a>,
    pub active_tab: LocalTabs,

    pub remote_size: TerminalSize,

    pub listeners: Vec<Listener>,
    pub listener_selection_index: usize,

    pub sessions: Vec<Arc<Session>>,
    pub highest_session_number: usize,
    pub session_selection_index: usize,
    pub session: Option<Arc<Session>>,

    pub network_bus: NetworkBus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

impl TerminalSize {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self { cols, rows }
    }
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self { cols: 80, rows: 24 }
    }
}

#[derive(Clone)]
pub struct NetworkBus {
    pub sender: Sender<NetworkEvent>,
    pub receiver: Receiver<NetworkEvent>,
}

impl Default for NetworkBus {
    fn default() -> Self {
        let (s, r) = unbounded();
        Self {
            sender: s,
            receiver: r,
        }
    }
}

pub enum NetworkEvent {
    NewSession(SessionBus),
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            tabs: TabsState {
                titles: vec!["Local", "Remote"],
                index: 0,
            },
            listeners: vec![],
            listener_selection_index: 0,
            active_tab: LocalTabs::Listeners,
            sessions: vec![],
            highest_session_number: 0,
            session_selection_index: 0,
            session: None,
            network_bus: Default::default(),

            // TODO: Change this to the actual terminal size
            remote_size: Default::default(),
        }
    }

    pub fn on_next(&mut self) {
        self.tabs.next();
    }

    pub fn on_tab(&mut self) {
        if self.tabs.index == 0 {
            self.active_tab = match self.active_tab {
                LocalTabs::Listeners => LocalTabs::Sessions,
                LocalTabs::Sessions => LocalTabs::Listeners,
            }
        }
    }

    pub fn on_create(&mut self) {
        if self.tabs.index == 0 && self.active_tab == LocalTabs::Listeners {
            let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], 1337)));

            match listener {
                Ok(listener) => {
                    self.listeners.push(Listener {
                        port: listener.local_addr().unwrap().port(),
                        status: ListenerStatus::Listening,
                    });

                    let network_bus = self.network_bus.clone();
                    thread::spawn(move || {
                        listen(listener, network_bus);
                    });
                }
                Err(_) => todo!(),
            }
        }
    }

    pub fn on_space(&mut self) {
        if self.tabs.index == 0 {
            match self.active_tab {
                LocalTabs::Sessions => {
                    let session = match self.sessions.get(self.session_selection_index) {
                        Some(session) => session,
                        None => return,
                    };

                    self.session = Some(session.clone());
                },
                LocalTabs::Listeners => {
                    
                }
            }
        }
    }

    pub fn on_enter(&mut self) {
        // todo!()
    }

    pub fn on_delete(&mut self) {
        if self.tabs.index == 0
            && self.active_tab == LocalTabs::Listeners
            && self.listener_selection_index < self.listeners.len()
        {
            self.listeners.remove(self.listener_selection_index);
        }
    }

    pub fn on_down(&mut self) {
        if self.tabs.index == 0 {
            match self.active_tab {
                LocalTabs::Listeners => {
                    if self.listener_selection_index < self.listeners.len() - 1 {
                        self.listener_selection_index += 1;
                    }
                },
                LocalTabs::Sessions => {
                    if self.session_selection_index < self.sessions.len() - 1 {
                        self.session_selection_index += 1;
                    }
                },
            }
        }
    }

    pub fn on_up(&mut self) {
        if self.tabs.index == 0 {
            match self.active_tab {
                LocalTabs::Listeners => {
                    if self.listener_selection_index > 0 {
                        self.listener_selection_index -= 1;
                    }
                },
                LocalTabs::Sessions => {
                    if self.session_selection_index > 0 {
                        self.session_selection_index -= 1;
                    }
                },
            }
        }
    }

    pub fn on_remote_interact(&mut self, input_key: KeyEvent) {
        // TODO: Fill these match arms!

        if input_key.kind != KeyEventKind::Press {
            return;
        }


        if let Some(session) = &self.session {
            let result = match input_key.code {
                KeyCode::Backspace => {
                    vec![8]
                },
                KeyCode::Enter => {
                    vec![b'\n']
                },
                KeyCode::Left => todo!(),
                KeyCode::Right => todo!(),
                KeyCode::Up => todo!(),
                KeyCode::Down => todo!(),
                KeyCode::Home => todo!(),
                KeyCode::End => todo!(),
                KeyCode::PageUp => todo!(),
                KeyCode::PageDown => todo!(),
                KeyCode::Tab => todo!(),
                KeyCode::BackTab => todo!(),
                KeyCode::Delete => todo!(),
                KeyCode::Insert => todo!(),
                KeyCode::F(_) => todo!(),
                KeyCode::Char(x) => {
                    vec![x as u8]
                },
                KeyCode::Null => todo!(),
                KeyCode::Esc => todo!(),
                KeyCode::CapsLock => todo!(),
                KeyCode::ScrollLock => todo!(),
                KeyCode::NumLock => todo!(),
                KeyCode::PrintScreen => todo!(),
                KeyCode::Pause => todo!(),
                KeyCode::Menu => todo!(),
                KeyCode::KeypadBegin => todo!(),
                KeyCode::Media(_) => todo!(),
                KeyCode::Modifier(_) => todo!(),
            };

            let _r = session.bus.s2c.sender.send(result);
        }
    }
}

#[derive(Debug, Display)]
#[strum(serialize_all = "UPPERCASE")]
pub enum ListenerStatus {
    Starting,
    Listening,
    Closed,
    Error,
}

pub struct Listener {
    pub port: u16,
    pub status: ListenerStatus,
}

#[derive(PartialEq, Debug)]
pub enum LocalTabs {
    Listeners,
    Sessions,
}

// TODO: Make this an enum, like LocalTabs
pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index == 0 {
            self.index = self.titles.len() - 1;
        } else {
            self.index -= 1;
        }
    }
}
