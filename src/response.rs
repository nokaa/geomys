#[derive(Clone)]
pub struct GoResponse {
    ftype: String,
    title: String,
    selector: String,
    domain: String,
    port: String,
}

impl GoResponse {
    pub fn new() -> GoResponse {
        GoResponse {
            ftype: String::new(),
            title: String::new(),
            selector: String::new(),
            domain: String::new(),
            port: String::new(),
        }
    }

    pub fn to_bytes(&mut self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        buf.append(&mut self.ftype.clone().into_bytes());
        buf.append(&mut self.title.clone().into_bytes());
        buf.push(b'\t');
        buf.append(&mut self.selector.clone().into_bytes());
        buf.push(b'\t');
        buf.append(&mut self.domain.clone().into_bytes());
        buf.push(b'\t');
        buf.append(&mut self.port.clone().into_bytes());
        buf.push(b'\r');
        buf.push(b'\n');

        buf
    }
}
