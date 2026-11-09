use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn from_stream(stream: &TcpStream) -> std::io::Result<Self> {
        let mut reader = BufReader::new(stream);

        // request line: METHOD PATH HTTP/x.y
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let mut parts = line.split_whitespace();
        let method = parts.next().unwrap_or("").to_string();
        let path = parts.next().unwrap_or("/").to_string();

        // headers until blank line
        let mut headers = HashMap::new();
        loop {
            let mut h = String::new();
            reader.read_line(&mut h)?;
            let h = h.trim_end();
            if h.is_empty() {
                break;
            }
            if let Some((k, v)) = h.split_once(':') {
                headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
            }
        }

        // read exactly Content-Length bytes, if any
        let body = match headers.get("content-length") {
            Some(len) => {
                let n: usize = len.parse().unwrap_or(0);
                let mut buf = vec![0u8; n];
                reader.read_exact(&mut buf)?;
                buf
            }
            None => Vec::new(),
        };

        Ok(Request {
            method,
            path,
            headers,
            body,
        })
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }

    pub fn content_type(&self) -> Option<&str> {
        self.header("content-type")
    }
}
