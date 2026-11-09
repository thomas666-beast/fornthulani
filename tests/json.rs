use fornthulani::{Params, Request, Response, Router, Server, Status, ThulaniError};
use serde::Serialize;
use std::net::TcpListener;
use std::thread;

#[derive(Serialize)]
struct User {
    id: u32,
    name: &'static str,
}

fn handler(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    Ok(Response::json(Status::OK, &User { id: 7, name: "ada" }))
}

#[test]
fn serves_json() {
    let mut router = Router::new();
    router.get("/user", handler);

    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);

    let (handle, join) = Server::new(router).spawn(&addr.to_string()).unwrap();
    thread::sleep(std::time::Duration::from_millis(50));

    let res = reqwest::blocking::get(format!("http://{}/user", addr)).unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers()["content-type"], "application/json");

    let body: serde_json::Value = res.json().unwrap();
    assert_eq!(body["id"], 7);
    assert_eq!(body["name"], "ada");

    handle.shutdown();
    join.join().unwrap();
}
