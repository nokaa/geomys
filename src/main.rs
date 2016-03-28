extern crate clap;
extern crate rotor;

mod gopher;
mod util;

use clap::App;
use rotor::{Loop, Config};
use rotor::mio::tcp::TcpListener;

use gopher::{Context, Gopher};

fn main() {
    let matches = App::new("geomys")
        .version("0.1")
        .about("An async gopher server")
        .args_from_usage(
            "-a, --addr=[ADDR] 'Sets the IP:PORT combination (default \"0.0.0.0:70\")'
            [ROOT] 'Sets the root dir (default \".\")'")
        .get_matches();

    let addr = matches.value_of("ADDR").unwrap_or("0.0.0.0:70");
    let root_dir = String::from(matches.value_of("ROOT").unwrap_or("."));

    println!("addr: {}", addr);
    println!("root dir: {}", root_dir);

    let context = Context {
        root_dir: root_dir,
    };

    let mut loop_creator = Loop::new(&Config::new()).unwrap();
    let lst = TcpListener::bind(&addr.parse().unwrap()).unwrap();
    loop_creator.add_machine_with(|scope| {
        Gopher::new(lst, scope)
    }).unwrap();
    loop_creator.run(context).unwrap();
}
