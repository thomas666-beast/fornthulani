//! Upload examples.
//!
//!   cargo run --example upload
//!
//!   curl -X POST --data-binary 'hello bytes' http://127.0.0.1:8081/raw
//!   curl -X POST -H 'Content-Type: application/json' \
//!        -d '{"name":"ada"}' http://127.0.0.1:8081/json
//!   curl -X POST --data-binary @Cargo.toml http://127.0.0.1:8081/save

use fornthulani::{Params, Request, Response, Router, Server, Status, ThulaniError};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct JsonIn {
    name: String,
}

// POST /raw  — echo bytes back
fn raw(req: &Request, _: &Params) -> Result<Response, ThulaniError> {
    Ok(Response::new(
        Status::OK,
        "application/octet-stream",
        req.body.clone(),
    ))
}

// POST /json  — parse JSON body
fn json_in(req: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let ct = req.content_type().unwrap_or("");
    if !ct.starts_with("application/json") {
        return Err(ThulaniError::BadRequest(format!("expected JSON, got {ct}")));
    }
    let payload: JsonIn = serde_json::from_slice(&req.body)
        .map_err(|e| ThulaniError::BadRequest(format!("bad json: {e}")))?;

    Ok(Response::text(Status::OK, &format!("hi {}", payload.name)))
}

// POST /save  — write body to a file
fn save(req: &Request, _: &Params) -> Result<Response, ThulaniError> {
    if req.body.is_empty() {
        return Err(ThulaniError::BadRequest("empty body".into()));
    }
    let path = PathBuf::from("upload.bin");
    std::fs::write(&path, &req.body)?;
    Ok(Response::text(
        Status::OK,
        &format!("wrote {} bytes to {}", req.body.len(), path.display()),
    ))
}

fn main() -> std::io::Result<()> {
    let mut router = Router::new();
    router.post("/raw", raw);
    router.post("/json", json_in);
    router.post("/save", save);

    println!("fornthulani upload examples on http://127.0.0.1:8081");
    println!("  POST /raw");
    println!("  POST /json   (Content-Type: application/json)");
    println!("  POST /save");

    Server::new(router).run_blocking("127.0.0.1:8081")
}
