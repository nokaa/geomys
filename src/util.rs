use std::fs::File;
use std::io::{self, Read};

pub fn local_path_for_request(request_path: &[u8], root_dir: &String) -> Option<Vec<u8>> {
    let mut index = false;
    // Check that request is a path
    if request_path[0] != b'/' {
        return None;
    }

    // Append the requested path to the root directory
    let mut path = root_dir.clone().into_bytes();
    let mut request_path = get_nonzero_bytes(request_path);
    request_path.pop(); // Remove LF
    request_path.pop(); // Remove CR
    path.append(&mut request_path);

    if path[path.len() - 1] == b'/' {
        path.append(&mut String::from("index.gph").into_bytes());
        index = true;
    }

    let path = String::from_utf8(path).unwrap();

    // Read file
    let mut f = match read_file(path) {
        Ok(f) => f,
        Err(_) => {
            return None
        }
    };

    if index {
        let len = f.len() - 1;
        if f[len] == b'\n' && f[len - 1] == b'\r' {
        } else if f[len] == b'\n' && f[len - 1] != b'\r' {
            f.insert(len, b'\r');
        } else {
            f.push(b'\r');
            f.push(b'\n');
        }
        f.push(b'.');
    }

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
