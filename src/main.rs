extern crate rotor;

mod gopher;
mod util;

use rotor::{Loop, Config};
use rotor::mio::tcp::TcpListener;

use gopher::{Context, Gopher};

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

