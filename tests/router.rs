use fornthulani::{Params, Request, Response, Router, Status, ThulaniError};
use std::collections::HashMap;

fn dummy_req(path: &str) -> Request {
    Request {
        method: "GET".into(),
        path: path.into(),
        headers: HashMap::new(),
        body: Vec::new(),
    }
}

fn hello(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    Ok(Response::text(Status::OK, "hello"))
}

#[test]
fn hits_route() {
    let mut r = Router::new();
    r.get("/hi", hello);
    match r.dispatch(&dummy_req("/hi")) {
        fornthulani::Body::Buffered(resp) => {
            assert_eq!(resp.status, Status::OK);
            assert_eq!(resp.body, b"hello");
        }
        _ => panic!("expected buffered response"),
    }
}

#[test]
fn misses_route() {
    let r = Router::new();
    match r.dispatch(&dummy_req("/nope")) {
        fornthulani::Body::Buffered(resp) => assert_eq!(resp.status, Status::NOT_FOUND),
        _ => panic!("expected buffered response"),
    }
}

#[test]
fn method_mismatch_yields_405() {
    let mut r = Router::new();
    r.get("/hi", hello);
    let mut req = dummy_req("/hi");
    req.method = "POST".into();
    match r.dispatch(&req) {
        fornthulani::Body::Buffered(resp) => assert_eq!(resp.status, Status::METHOD_NOT_ALLOWED),
        _ => panic!("expected buffered response"),
    }
}

#[test]
fn param_route_matches() {
    fn show(_: &Request, params: &Params) -> Result<Response, ThulaniError> {
        Ok(Response::text(
            Status::OK,
            &format!("id={}", params.require("id")?),
        ))
    }
    let mut r = Router::new();
    r.get("/user/:id", show);
    match r.dispatch(&dummy_req("/user/42")) {
        fornthulani::Body::Buffered(resp) => {
            assert_eq!(resp.status, Status::OK);
            assert_eq!(resp.body, b"id=42");
        }
        _ => panic!("expected buffered response"),
    }
}
