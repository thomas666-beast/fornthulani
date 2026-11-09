//! Small end-to-end example for `fornthulani`.
//!
//! Run with:
//!     cargo run --example hello
//!
//! Then in another terminal:
//!     curl http://127.0.0.1:8080/hello
//!     curl http://127.0.0.1:8080/user/42
//!     curl http://127.0.0.1:8080/user/0
//!     curl -o download.toml http://127.0.0.1:8080/download

use fornthulani::{Params, Request, Response, Router, Server, Status, ThulaniError};
use serde::Serialize;

#[derive(Serialize)]
struct User {
    id: u32,
    name: &'static str,
}

// GET /hello
fn hello(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    Ok(Response::text(Status::OK, "hello, fornthulani"))
}

// GET /user/:id  — JSON, with a param
fn show_user(_: &Request, params: &Params) -> Result<Response, ThulaniError> {
    let raw = params.require("id")?;
    let id: u32 = raw
        .parse()
        .map_err(|_| ThulaniError::BadRequest(format!("id `{raw}` is not a number")))?;

    if id == 0 {
        return Err(ThulaniError::NotFound("no user 0".into()));
    }

    Ok(Response::json(Status::OK, &User { id, name: "ada" }))
}

// GET /msgpack  — same payload, MessagePack
fn show_msgpack(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    Ok(Response::msgpack(Status::OK, &User { id: 7, name: "ada" }))
}

// GET /download  — reads the file into a Response (demo only).
// Real streaming lives in `FileStream` + `Body::Stream`, which needs
// a `stream_get` registration helper on Router (planned, not in v0.1).
fn download(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let body = std::fs::read("Cargo.toml")?;
    Ok(Response::new(Status::OK, "text/plain; charset=utf-8", body))
}

fn main() -> std::io::Result<()> {
    let mut router = Router::new();
    router.get("/hello", hello);
    router.get("/user/:id", show_user);
    router.get("/msgpack", show_msgpack);
    router.get("/download", download);

    println!("fornthulani listening on http://127.0.0.1:8080");
    println!("  GET /hello");
    println!("  GET /user/42");
    println!("  GET /user/0        (404)");
    println!("  GET /user/abc      (400)");
    println!("  GET /msgpack");
    println!("  GET /download");

    Server::new(router).run_blocking("127.0.0.1:8080")
}
