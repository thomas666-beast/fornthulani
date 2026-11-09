use fornthulani::{Params, Request, Response, Router, Server, Status, ThulaniError};
use std::net::TcpListener;
use std::thread;

#[test]
fn reads_post_body() {
    fn echo(req: &Request, _: &Params) -> Result<Response, ThulaniError> {
        Ok(Response::new(
            Status::OK,
            "application/octet-stream",
            req.body.clone(),
        ))
    }

    let mut router = Router::new();
    router.post("/echo", echo);

    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);

    let (handle, join) = Server::new(router).spawn(&addr.to_string()).unwrap();
    thread::sleep(std::time::Duration::from_millis(50));

    let res = reqwest::blocking::Client::new()
        .post(format!("http://{}/echo", addr))
        .body("hello upload")
        .send()
        .unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(&res.bytes().unwrap()[..], b"hello upload");

    handle.shutdown();
    join.join().unwrap();
}
