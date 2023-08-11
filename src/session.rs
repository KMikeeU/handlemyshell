use std::{net::{TcpListener, TcpStream}, sync::{Arc, Mutex}, thread, io::{Read, Write}};

use crossbeam::channel::{bounded, unbounded, Sender, Receiver};

use crate::app::{App, NetworkEvent, NetworkBus};


pub struct Session {
    pub id: usize,
    pub streams: SessionStreams,
}

#[derive(Clone)]
pub struct SessionStreams {
    pub sender: Sender<Vec<u8>>,
    pub receiver: Receiver<Vec<u8>>,
}

impl SessionStreams {
    pub fn new() -> Self {
        let (s,r) = unbounded();
        Self {
            sender: s,
            receiver: r,
        }
    }
}


pub fn handle_connection(session: SessionStreams, stream: TcpStream) {
    let mut write_stream = stream.try_clone().unwrap();

    thread::spawn(move || {
        let mut stream = stream;
        let mut buf = [0; 1024];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => {
                    break;
                }
                Ok(n) => {
                    let cl = buf.to_vec();
                    session.sender.send(cl).unwrap();
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }

            
        }
    });

    thread::spawn(move || {
        for i in 0..session.receiver.len() {
            let mut buf = session.receiver.recv().expect("recv from channel failed, eventhough the length has been checked");
            write_stream.write(&mut buf).unwrap();
        }
    });


}


pub fn listen(mut listener: TcpListener, network_bus: NetworkBus) {

    loop {
        let stream = listener.accept();

        match stream {
            Ok((stream, addr)) => {
                let session = SessionStreams::new();
                
                network_bus.sender.send(NetworkEvent::NewSession(session.clone())).unwrap();


                handle_connection(session.clone(), stream);
            }
            Err(e) => {
            }
        }
    }
}