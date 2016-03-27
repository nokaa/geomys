extern crate rotor;

use std::fs::File;
use std::io::{self, Read, Write, stderr};
use std::path::{Path, PathBuf};
use std::str;

use rotor::{EventSet, PollOpt, Loop, Config, Void};
use rotor::mio::{TryRead, TryWrite};
use rotor::mio::tcp::{TcpListener, TcpStream};
use rotor::{Machine, Response, EarlyScope, Scope};

mod response;


struct Context {
    root_dir: String,
}

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
                        let data_str = str::from_utf8(&data).unwrap();
                        println!("{}", data_str);
                        match local_path_for_request(&data, &scope.root_dir) {
                            Some(p) => {
                                match sock.try_write(&p) {
                                    Ok(_) => Response::ok(Gopher::Connection(sock)),
                                    Err(e) => {
                                        writeln!(&mut stderr(), "write: {}", e).ok();
                                        Response::done()
                                    }
                                }
                            }
                            None => Response::done(),
                        }
                        //Response::done()
                        /*match sock.try_write(&data[..x]) {
                            Ok(_) => {
                                // this is example so we don't care if not all
                                // (or none at all) bytes are written
                                Response::ok(Gopher::Connection(sock))
                            }
                            Err(e) => {
                                writeln!(&mut stderr(), "write: {}", e).ok();
                                Response::done()
                            }
                        }*/
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
    let root_dir = String::from(".");
    let context = Context {
        root_dir: root_dir,
    };

    let mut loop_creator = Loop::new(&Config::new()).unwrap();
    let lst = TcpListener::bind(&"127.0.0.1:3000".parse().unwrap()).unwrap();
    loop_creator.add_machine_with(|scope| {
        Gopher::new(lst, scope)
    }).unwrap();
    loop_creator.run(context).unwrap();
}

fn local_path_for_request(request_path: &[u8], root_dir: &String) -> Option<Vec<u8>> {
    // Check that request is a path
    if request_path[0] != b'/' {
        return None;
    }

    // Append the requested path to the root directory
    let mut path = root_dir.clone().into_bytes();
    let mut request_path = get_nonzero_bytes(request_path);
    request_path.pop();
    path.append(&mut request_path);

    if path[path.len() - 1] == b'/' {
        path.append(&mut String::from("index.gph").into_bytes());
    }

    let path = String::from_utf8(path).unwrap();

    // Read file
    let f = match read_file(path) {
        Ok(f) => f,
        Err(_) => {
            return None
        }
    };

    Some(f)
}

fn read_file(filename: String) -> Result<Vec<u8>, io::Error> {
    let mut f = try!(File::open(filename));
    let mut buf: Vec<u8> = vec![];
    try!(f.read_to_end(&mut buf));

    Ok(buf)
}

fn get_nonzero_bytes(data: &[u8]) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![];
    for &ch in data {
        if ch == 0u8 {
            continue;
        } else {
            buf.push(ch);
        }
    }

    buf
}
