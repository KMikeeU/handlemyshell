use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread, sync::Mutex,
};

use crossbeam::channel::{unbounded, Receiver, Sender};

use crate::app::{NetworkBus, NetworkEvent};

pub struct Session {
    pub id: usize,
    pub bus: SessionBus,
    pub term: Mutex<vt100::Parser>,
}

#[derive(Clone)]
pub struct SessionStreams {
    pub sender: Sender<Vec<u8>>,
    pub receiver: Receiver<Vec<u8>>,
}

#[derive(Clone)]
pub struct SessionBus {
    pub c2s: SessionStreams,
    pub s2c: SessionStreams,
}
impl SessionBus {
    pub fn new() -> Self {
        Self {
            c2s: SessionStreams::new(),
            s2c: SessionStreams::new(),
        }
    }
}

impl SessionStreams {
    pub fn new() -> Self {
        let (s, r) = unbounded();
        Self {
            sender: s,
            receiver: r,
        }
    }
}

pub fn handle_connection(session: SessionBus, stream: TcpStream) {
    let mut write_stream = stream.try_clone().unwrap();

    // Read from the socket and send to the channel
    thread::spawn(move || {
        let mut stream = stream;
        let mut buf = [0; 1024];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => {
                    break;
                }
                Ok(_n) => {
                    let cl = buf.to_vec();
                    session.c2s.sender.send(cl).unwrap();
                }
                Err(e) => {
                    break;
                }
            }
        }
    });

    // Read from the channel and write to the socket
    thread::spawn(move || {
        while let Ok(buf) = session.s2c.receiver.recv() {
            write_stream.write_all(&buf).unwrap();
        }
        // TODO: What happens when the channel is closed?
    });
}

pub fn listen(listener: TcpListener, network_bus: NetworkBus) {
    loop {
        let stream = listener.accept();

        match stream {
            Ok((stream, _addr)) => {
                let session = SessionBus::new();

                network_bus
                    .sender
                    .send(NetworkEvent::NewSession(session.clone()))
                    .unwrap();

                handle_connection(session.clone(), stream);
            }
            Err(_e) => {}
        }
    }
}
