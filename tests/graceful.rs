use fornthulani::{Params, Request, Response, Router, Server, Status, ThulaniError};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

fn hi(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    Ok(Response::text(Status::OK, "hello"))
}

#[test]
fn shuts_down_cleanly() {
    let mut router = Router::new();
    router.get("/hi", hi);

    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);

    let (handle, join) = Server::new(router).spawn(&addr.to_string()).unwrap();
    thread::sleep(Duration::from_millis(50));

    let res = reqwest::blocking::get(format!("http://{}/hi", addr)).unwrap();
    assert_eq!(res.status(), 200);

    handle.shutdown();
    join.join().unwrap();

    TcpListener::bind(addr).expect("port should be free after shutdown");
}
