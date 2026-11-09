use fornthulani::{Params, Request, Response, Router, Server, Status, ThulaniError};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::thread;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Item {
    sku: String,
    qty: u32,
}

fn handler(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    Ok(Response::msgpack(
        Status::OK,
        &Item {
            sku: "ABC".into(),
            qty: 3,
        },
    ))
}

#[test]
fn serves_msgpack() {
    let mut router = Router::new();
    router.get("/item", handler);

    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);

    let (handle, join) = Server::new(router).spawn(&addr.to_string()).unwrap();
    thread::sleep(std::time::Duration::from_millis(50));

    let res = reqwest::blocking::get(format!("http://{}/item", addr)).unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers()["content-type"], "application/msgpack");

    let bytes = res.bytes().unwrap();
    let item: Item = rmp_serde::from_slice(&bytes).unwrap();
    assert_eq!(
        item,
        Item {
            sku: "ABC".into(),
            qty: 3
        }
    );

    handle.shutdown();
    join.join().unwrap();
}
