use std::{net::{TcpListener, SocketAddr, TcpStream}, sync::{Arc, Mutex}, thread, rc::Rc};

use crossbeam::channel::{Sender, Receiver, unbounded};
use strum_macros::Display;

use crate::session::{SessionStreams, listen, Session};






pub struct App<'a> {
    pub tabs: TabsState<'a>,
    pub active_tab: LocalTabs,

    pub listeners: Vec<Listener>,
    pub listener_selection_index: usize,

    pub sessions: Vec<Arc<Session>>,
    pub highest_session_number: usize,
    pub session_selection_index: usize,
    pub session: Option<Arc<Session>>,

    pub network_bus: NetworkBus
}

#[derive(Clone)]
pub struct NetworkBus {
    pub sender: Sender<NetworkEvent>,
    pub receiver: Receiver<NetworkEvent>
}

impl Default for NetworkBus {
    fn default() -> Self {
        let (s,r) = unbounded();
        Self {
            sender: s,
            receiver: r,
        }
    }
}

pub enum NetworkEvent { 
    NewSession(SessionStreams),
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
        if self.tabs.index == 0 {
            if self.active_tab == LocalTabs::Listeners {
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
                        


                    },
                    Err(_) => todo!(),
                }

            }
        }
    }

    pub fn on_space(&mut self) {
        if self.tabs.index == 0 {
            if self.active_tab == LocalTabs::Sessions {
                let session = match self.sessions.get(self.session_selection_index) {
                    Some(session) => session,
                    None => return,
                };

                self.session = Some(session.clone());
            }
        }
    }

    pub fn on_enter(&mut self) {
        todo!()
    }

    pub fn on_delete(&mut self) {
        if self.tabs.index == 0 {
            if self.active_tab == LocalTabs::Listeners {
                if self.listener_selection_index < self.listeners.len() {
                    self.listeners.remove(self.listener_selection_index);
                }
            }
        }
    }

    pub fn on_down(&mut self) {
        if self.tabs.index == 0 {
            if self.active_tab == LocalTabs::Listeners {
                if self.listener_selection_index < self.listeners.len() - 1 {
                    self.listener_selection_index += 1;
                }
            }
        }
    }

    pub fn on_up(&mut self) {
        if self.tabs.index == 0 {
            if self.active_tab == LocalTabs::Listeners {
                if self.listener_selection_index > 0 {
                    self.listener_selection_index -= 1;
                }
            }
        }
    }
}


#[derive(Debug, Display)]
#[strum(serialize_all = "UPPERCASE")]
pub enum ListenerStatus {
    Starting,
    Listening,
    Closed,
    Error
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


