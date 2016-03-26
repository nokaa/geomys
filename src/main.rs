extern crate rotor;

use std::io::{Write, stderr};

use rotor::{EventSet, PollOpt, Loop, Config, Void};
use rotor::mio::{TryRead, TryWrite};
use rotor::mio::tcp::{TcpListener, TcpStream};
use rotor::{Machine, Response, EarlyScope, Scope};


struct Context;

enum Gopher {
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

    fn ready(self, _events: EventSet, _scope: &mut Scope<Context>)
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
                    Ok(Some(x)) => {
                        match sock.try_write(&data[..x]) {
                            Ok(_) => {
                                // this is example so we don't care if not all
                                // (or none at all) bytes are written
                                Response::ok(Gopher::Connection(sock))
                            }
                            Err(e) => {
                                writeln!(&mut stderr(), "write: {}", e).ok();
                                Response::done()
                            }
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

fn main() {
    let mut loop_creator = Loop::new(&Config::new()).unwrap();
    let lst = TcpListener::bind(&"127.0.0.1:3000".parse().unwrap()).unwrap();
    loop_creator.add_machine_with(|scope| {
        Gopher::new(lst, scope)
    }).unwrap();
    loop_creator.run(Context).unwrap();
}
