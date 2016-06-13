/* Copyright (C)  2016 nokaa <nokaa@cock.li>
 * This software is licensed under the terms of the
 * GNU Affero General Public License. You should have
 * received a copy of this license with this software.
 * The license may also be found at https://gnu.org/licenses/agpl.txt
 */

use std::io::{Write, stderr};
use std::str;

use rotor::{EventSet, PollOpt, Void};
use rotor::mio::{TryRead, TryWrite};
use rotor::mio::tcp::{TcpListener, TcpStream};
use rotor::{Machine, Response, EarlyScope, Scope};

use ::util::*;

pub struct Context {
    pub root_dir: String,
    pub counter: u64,
}

pub trait Counter {
    fn increment(&mut self);
    fn get(&self) -> u64;
}

impl Counter for Context {
    fn increment(&mut self) {
        // To prevent crashing on overflow, we simply wrap back to 0.
        self.counter += 1;
    }

    fn get(&self) -> u64 {
        self.counter
    }
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
        scope.increment();
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
                    Ok(Some(x)) => {
                        match local_path_for_request(&data[..x], scope) {
                            Some(p) => {
                                let mut b = 0; // The number of bytes written so far.
                                while b < p.len() {
                                    match sock.try_write(&p[b..]) {
                                        Ok(Some(x)) => {
                                            b += x
                                        }
                                        Ok(None) => {
                                            return Response::done();
                                        }
                                        Err(e) => {
                                            writeln!(&mut stderr(), "write: {}", e).ok();
                                            return Response::done();
                                        }
                                    }
                                }

                                Response::done()
                            }
                            None => {
                                let token = match str::from_utf8(&data[..x-2]) {
                                    Ok(t) => t,
                                    Err(_) => "INVALID",
                                };

                                let d = &format!("3Sorry, but the requested token '{}' could not be found.\tErr\tlocalhost\t70\r\n.", token)[..];
                                match sock.try_write(d.as_bytes()) {
                                    Ok(_) => Response::done(),
                                    Err(e) => {
                                        writeln!(&mut stderr(), "write: {}", e).ok();
                                        Response::done()
                                    }
                                }
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
