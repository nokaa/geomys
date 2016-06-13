/* Copyright (C)  2016 nokaa <nokaa@cock.li>
 * This software is licensed under the terms of the
 * GNU Affero General Public License. You should have
 * received a copy of this license with this software.
 * The license may also be found at https://gnu.org/licenses/agpl.txt
 */

#[macro_use]
extern crate clap;
extern crate rotor;
extern crate rustc_serialize;
extern crate toml;

mod gopher;
mod util;

use clap::{App, Arg};
use rotor::{Loop, Config};
use rotor::mio::tcp::TcpListener;
use rustc_serialize::Decoder;

use gopher::{Context, Gopher};

use std::{env, fs, process};
use std::io::Read;

fn main() {
    let matches = App::new("geomys")
        .version(crate_version!())
        .author("nokaa <nokaa@cock.li>")
        .about("An async gopher server")
        .arg(Arg::with_name("addr")
             .short("a")
             .long("addr")
             .value_name("ADDR")
             .help("Sets the IP:PORT combination (default \"0.0.0.0:70\")")
             .takes_value(true))
        .arg(Arg::with_name("ROOT")
             .help("Sets the root directory (default \"/var/gopher\")")
             .required(false)
             .index(1))
        .get_matches();

    // Get the user's home directory for attempting to read
    // ~/.config/geomys/config
    let mut home = env::home_dir().unwrap();
    home.push(".config/geomys/config");
    let home = home.to_str().unwrap();

    let addr: String;
    let root_dir: String;

    if let Some(a) = matches.value_of("addr") {
        addr = String::from(a);
    } else if let Some(a) = val_exists(home, String::from("address")) {
        println!("reading address from {}", home);
        addr = a;
    } else if let Some(a) = val_exists("/etc/geomys/config", String::from("address")) {
        println!("reading address from /etc/geomys/config");
        addr = a;
    } else {
        println!("using default value for addr");
        addr = String::from("0.0.0.0:70");
    }

    if let Some(dir) = matches.value_of("ROOT") {
        root_dir = String::from(dir);
    } else if let Some(dir) = val_exists(home, String::from("root_dir")) {
        println!("reading root_dir from {}", home);
        root_dir = dir;
    } else if let Some(dir) = val_exists("/etc/geomys/config", String::from("root_dir")) {
        println!("reading root_dir from /etc/geomys/config");
        root_dir = dir;
    } else {
        println!("using default value for root_dir");
        root_dir = String::from("/var/gopher");
    }

    println!("addr: {}", addr);
    println!("root dir: {}", root_dir);

    let context = Context {
        root_dir: root_dir,
        counter: 0,
    };

    let mut loop_creator = Loop::new(&Config::new()).unwrap();
    let lst = match TcpListener::bind(&addr.parse().unwrap()) {
        Ok(l) => l,
        Err(e) => {
            println!("{}", e);
            println!("Note that running on a port below 1000 requires root");
            process::exit(1);
        }
    };
    loop_creator.add_machine_with(|scope| {
        Gopher::new(lst, scope)
    }).unwrap();
    loop_creator.run(context).unwrap();
}

fn val_exists(filename: &str, value: String) -> Option<String> {
    if !file_exists(filename) {
        return None;
    }

    let mut f = fs::File::open(filename).unwrap();
    let mut tom = String::new();
    match f.read_to_string(&mut tom) {
        Ok(_) => { },
        Err(_) => return None,
    }

    let mut parser = toml::Parser::new(&tom);
    let table = match parser.parse() {
        Some(t) => t,
        None => return None,
    };

    match table.get(&value) {
        Some(val) => match toml::Decoder::new(val.clone()).read_str() {
            Ok(s) => return Some(s),
            Err(_) => return None,
        },
        None => return None,
    }
}

fn file_exists(filename: &str) -> bool {
    match fs::File::open(filename) {
        Ok(_) => true,
        Err(_) => false,
    }
}
