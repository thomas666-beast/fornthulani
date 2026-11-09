use fornthulani::{FileStream, Params, Request, Response, Router, Server, Status, ThulaniError};
use std::net::TcpListener;
use std::sync::OnceLock;
use std::thread;

static FILE_PATH: OnceLock<std::path::PathBuf> = OnceLock::new();

fn serve(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let body = std::fs::read(FILE_PATH.get().unwrap())?;
    Ok(Response::new(Status::OK, "text/plain", body))
}

#[test]
fn serves_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("hello.txt");
    std::fs::write(&file_path, b"file contents").unwrap();
    let _ = FILE_PATH.set(file_path);

    let mut router = Router::new();
    router.get("/download", serve);

    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);

    let (handle, join) = Server::new(router).spawn(&addr.to_string()).unwrap();
    thread::sleep(std::time::Duration::from_millis(50));

    let res = reqwest::blocking::get(format!("http://{}/download", addr)).unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers()["content-type"], "text/plain");
    assert_eq!(&res.bytes().unwrap()[..], b"file contents");

    handle.shutdown();
    join.join().unwrap();
}

#[test]
fn file_stream_open_works() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("payload.bin");
    std::fs::write(&file_path, b"\x00\x01\x02").unwrap();
    let stream = FileStream::open(&file_path).unwrap();
    assert_eq!(stream.size, 3);
    assert_eq!(stream.content_type, "application/octet-stream");
}
