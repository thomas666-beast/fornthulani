use fornthulani::request::Request;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;

#[test]
fn parses_basic_get() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        Request::from_stream(&stream).unwrap()
    });

    let mut client = TcpStream::connect(addr).unwrap();
    client
        .write_all(b"GET /hello HTTP/1.1\r\nHost: x\r\nFoo: bar\r\n\r\n")
        .unwrap();
    client.shutdown(std::net::Shutdown::Write).unwrap();

    let req = handle.join().unwrap();
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, "/hello");
    assert_eq!(req.headers.get("foo").map(String::as_str), Some("bar"));
}
