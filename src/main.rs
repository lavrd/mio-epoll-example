use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    str::from_utf8,
    thread,
};

use log::{debug, info, trace};
use mio::{net::TcpListener, Events, Interest, Poll, Token};
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

const SERVER: Token = Token(0);

fn main() {
    env_logger::init();
    info!("starting up");

    thread::spawn(move || loop {
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(128);

        let addr: std::net::SocketAddr = "127.0.0.1:6969".parse().unwrap();
        let mut server = TcpListener::bind(addr).unwrap();

        poll.registry()
            .register(&mut server, SERVER, Interest::READABLE)
            .unwrap();

        let mut connections = HashMap::new();
        let mut next_token = 1;

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                trace!("new event: {:?}", event);

                match event.token() {
                    SERVER => {
                        let (mut socket, addr) = server.accept().unwrap();
                        debug!("new client: {:?}", addr);

                        let token = Token(next_token);
                        next_token += 1;
                        poll.registry()
                            .register(&mut socket, token, Interest::READABLE)
                            .unwrap();

                        connections.insert(token, socket);
                    }
                    token => {
                        let conn = connections.get_mut(&token).unwrap();
                        let addr = conn.peer_addr();
                        if event.is_readable() {
                            let mut buf: Vec<u8> = vec![0; 1024];
                            let n = conn.read(&mut buf);
                            match n {
                                Ok(0) => {
                                    debug!("connection closed: {:?}", addr);
                                    connections.remove(&token);
                                }
                                Ok(n) => {
                                    debug!("received data: {:?}", from_utf8(&buf[..n]).unwrap());
                                    conn.write_all(&buf[0..n]).unwrap();
                                }
                                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                    // This error means that in socket buffer there are not data but it is not closed.
                                }
                                Err(e) => {
                                    panic!("failed to read from connection: {:?}: {:?}", addr, e)
                                }
                            }
                        }
                        if event.is_writable() {
                            unimplemented!()
                        }
                    }
                }
            }
        }
    });

    let mut sigs = Signals::new(TERM_SIGNALS).unwrap();
    let sig = sigs.into_iter().next().unwrap();
    debug!("received term signal: {:?}", sig);
}
