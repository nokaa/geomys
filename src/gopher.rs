use std::io::{Write, stderr};
use std::str;

use rotor::{EventSet, PollOpt, Void};
use rotor::mio::{TryRead, TryWrite};
use rotor::mio::tcp::{TcpListener, TcpStream};
use rotor::{Machine, Response, EarlyScope, Scope};

use super::util::*;

pub struct Context {
    pub root_dir: String,
}

pub enum Gopher {
    Server(TcpListener),
    Connection(TcpStream),
}

impl Gopher {
    pub fn new(sock: TcpListener, scope: &mut EarlyScope)
        -> Response<Gopher, Void>
    {
        scope.register(&sock, EventSet::readable(), PollOpt::edge())
            .unwrap();
        Response::ok(Gopher::Server(sock))
    }
    fn accept(self) -> Response<Gopher, TcpStream> {
        match self {
            Gopher::Server(sock) => {
                match sock.accept() {
                    Ok(Some((conn, _))) => {
                        Response::spawn(Gopher::Server(sock), conn)
                    }
                    Ok(None) => {
                        Response::ok(Gopher::Server(sock))
                    }
                    Err(e) => {
                        writeln!(&mut stderr(), "Error: {}", e).ok();
                        Response::ok(Gopher::Server(sock))
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Machine for Gopher {
    type Context = Context;
    type Seed = TcpStream;

    fn create(conn: TcpStream, scope: &mut Scope<Context>)
        -> Response<Self, Void>
    {
        scope.register(&conn, EventSet::readable(), PollOpt::level())
            .unwrap();
        Response::ok(Gopher::Connection(conn))
    }

    fn ready(self, _events: EventSet, scope: &mut Scope<Context>)
        -> Response<Self, TcpStream>
    {
        match self {
            me @ Gopher::Server(..) => me.accept(),
            Gopher::Connection(mut sock) => {
                let mut data = [0u8; 1024];
                match sock.try_read(&mut data) {
                    Err(e) => {
                        writeln!(&mut stderr(), "read: {}", e).ok();
                        Response::done()
                    }
                    Ok(Some(0)) => {
                        Response::done()
                    }
                    Ok(Some(_)) => {
                        let data_str = str::from_utf8(&data).unwrap();
                        println!("{}", data_str);
                        match local_path_for_request(&data, &scope.root_dir) {
                            Some(p) => {
                                match sock.try_write(&p) {
                                    Ok(_) => Response::done(),
                                    //Ok(_) => Response::ok(Gopher::Connection(sock)),
                                    Err(e) => {
                                        writeln!(&mut stderr(), "write: {}", e).ok();
                                        Response::done()
                                    }
                                }
                            }
                            None => Response::done(),
                        }
                    }
                    Ok(None) => {
                        Response::ok(Gopher::Connection(sock))
                    }
                }
            }
        }
    }
    fn spawned(self, _scope: &mut Scope<Context>) -> Response<Self, TcpStream>
    {
        match self {
            me @ Gopher::Server(..) => me.accept(),
            _ => unreachable!(),
        }
    }
    fn timeout(self, _scope: &mut Scope<Context>)
        -> Response<Self, TcpStream>
    {
        unreachable!();
    }
    fn wakeup(self, _scope: &mut Scope<Context>)
        -> Response<Self, TcpStream>
    {
        unreachable!();
    }
}
