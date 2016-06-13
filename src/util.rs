/* Copyright (C)  2016 nokaa <nokaa@cock.li>
 * This software is licensed under the terms of the
 * GNU Affero General Public License. You should have
 * received a copy of this license with this software.
 * The license may also be found at https://gnu.org/licenses/agpl.txt
 */

use rotor::Scope;

use gopher::{Context, Counter};

use std::fs::File;
use std::io::{self, Read};

pub fn local_path_for_request(request_path: &[u8], scope: &Scope<Context>) -> Option<Vec<u8>> {
    let mut index = false;

    // Append the requested path to the root directory
    let mut path = scope.root_dir.clone().into_bytes();
    let mut request_path = request_path.to_vec();
    request_path.pop(); // Remove LF
    request_path.pop(); // Remove CR

    if request_path == vec![] {
        path.push(b'/');
    } else if request_path == vec![b'/', b'v', b'i', b's', b'i', b't', b's'] {
        let f = format!("This host has been visited {} times", scope.get())
            .into_bytes();
        return Some(f);
    } else {
        path.append(&mut request_path);
    }

    if path[path.len() - 1] == b'/' {
        path.append(&mut String::from("index.gph").into_bytes());
        index = true;
    }

    let path = match String::from_utf8(path) {
        Ok(s) => s,
        Err(_) => return None,
    };

    // Read file
    let mut f: Vec<u8>;
    if index {
        f = match read_index(path) {
            Ok(f) => f,
            Err(_) => return None,
        };
        f.push(b'.');
        f.push(b'\r');
        f.push(b'\n');
    } else {
        f = match read_file(path) {
            Ok(f) => f,
            Err(_) => return None,
        };
    }

    Some(f)
}

fn read_file(filename: String) -> Result<Vec<u8>, io::Error> {
    let mut f = try!(File::open(filename));
    let mut buf: Vec<u8> = vec![];
    try!(f.read_to_end(&mut buf));

    Ok(buf)
}

fn read_index(filename: String) -> Result<Vec<u8>, io::Error> {
    let f = try!(File::open(filename));
    let mut buf: Vec<u8> = vec![];

    for byte in f.bytes() {
        match byte {
            Err(e) => return Err(e),
            Ok(b) => match b {
                b'\n' => {
                    buf.push(b'\r');
                    buf.push(b);
                }
                _ => buf.push(b),
            }
        }
    }

    Ok(buf)
}
