use crate::status::Status;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct Response {
    pub status: Status,
    pub content_type: String,
    pub body: Vec<u8>,
    pub extra_headers: Vec<(String, String)>,
}

impl Response {
    pub fn new(status: Status, content_type: &str, body: Vec<u8>) -> Self {
        Self {
            status,
            content_type: content_type.to_string(),
            body,
            extra_headers: Vec::new(),
        }
    }

    pub fn text(status: Status, body: &str) -> Self {
        Self::new(
            status,
            "text/plain; charset=utf-8",
            body.as_bytes().to_vec(),
        )
    }

    pub fn json<T: serde::Serialize>(status: Status, value: &T) -> Self {
        let body = serde_json::to_vec(value).unwrap_or_else(|_| b"null".to_vec());
        Self::new(status, "application/json", body)
    }

    pub fn msgpack<T: serde::Serialize>(status: Status, value: &T) -> Self {
        let body = rmp_serde::to_vec_named(value).unwrap_or_default();
        Self::new(status, "application/msgpack", body)
    }

    pub fn header(mut self, k: &str, v: &str) -> Self {
        self.extra_headers.push((k.to_string(), v.to_string()));
        self
    }

    pub fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        write!(w, "HTTP/1.1 {}\r\n", self.status)?;
        write!(w, "Content-Type: {}\r\n", self.content_type)?;
        write!(w, "Content-Length: {}\r\n", self.body.len())?;
        write!(w, "Connection: close\r\n")?;
        for (k, v) in &self.extra_headers {
            write!(w, "{}: {}\r\n", k, v)?;
        }
        write!(w, "\r\n")?;
        w.write_all(&self.body)?;
        w.flush()
    }
}

// ----- streaming files -----

pub struct FileStream {
    pub path: PathBuf,
    pub content_type: String,
    pub size: u64,
    pub filename: String,
}

impl FileStream {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let meta = std::fs::metadata(&path)?;
        if !meta.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path is not a regular file",
            ));
        }
        let content_type = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .essence_str()
            .to_string();
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("download")
            .to_string();
        Ok(Self {
            path,
            content_type,
            size: meta.len(),
            filename,
        })
    }

    pub fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        write!(w, "HTTP/1.1 200 OK\r\n")?;
        write!(w, "Content-Type: {}\r\n", self.content_type)?;
        write!(w, "Content-Length: {}\r\n", self.size)?;
        write!(
            w,
            "Content-Disposition: attachment; filename=\"{}\"\r\n",
            self.filename
        )?;
        write!(w, "Connection: close\r\n")?;
        write!(w, "\r\n")?;
        let mut f = File::open(&self.path)?;
        io::copy(&mut f, w)?;
        w.flush()
    }
}

// ----- unified body -----

pub enum Body {
    Buffered(Response),
    Stream(FileStream),
}

impl Body {
    pub fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        match self {
            Body::Buffered(r) => r.write_to(w),
            Body::Stream(s) => s.write_to(w),
        }
    }
}

impl From<Response> for Body {
    fn from(r: Response) -> Self {
        Body::Buffered(r)
    }
}

impl From<FileStream> for Body {
    fn from(s: FileStream) -> Self {
        Body::Stream(s)
    }
}
