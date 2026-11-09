use fornthulani::{Middleware, Params, Request, Response, Router, Server, Status, ThulaniError};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

fn hello(_: &Request, _: &Params) -> Result<Response, ThulaniError> {
    Ok(Response::text(Status::OK, "hello"))
}

struct Counter(Arc<AtomicUsize>);

impl Middleware for Counter {
    fn before(&self, _: &mut Request) -> Result<Option<Response>, ThulaniError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(None)
    }
}

struct AddHeader;

impl Middleware for AddHeader {
    fn after(&self, _: &Request, resp: &mut Response) {
        resp.extra_headers
            .push(("X-Powered-By".into(), "fornthulani".into()));
    }
}

struct ShortCircuit;

impl Middleware for ShortCircuit {
    fn before(&self, _: &mut Request) -> Result<Option<Response>, ThulaniError> {
        Ok(Some(Response::text(Status::OK, "short")))
    }
}

struct Reject;

impl Middleware for Reject {
    fn before(&self, _: &mut Request) -> Result<Option<Response>, ThulaniError> {
        Err(ThulaniError::Forbidden("nope".into()))
    }
}

#[test]
fn global_middleware_runs() {
    let counter = Arc::new(AtomicUsize::new(0));

    let mut router = Router::new();
    router.get("/hi", hello);

    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);

    let (handle, join) = Server::new(router)
        .with_middleware(Counter(Arc::clone(&counter)))
        .with_middleware(AddHeader)
        .spawn(&addr.to_string())
        .unwrap();
    thread::sleep(std::time::Duration::from_millis(50));

    let res = reqwest::blocking::get(format!("http://{}/hi", addr)).unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers()["x-powered-by"], "fornthulani");
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    handle.shutdown();
    join.join().unwrap();
}

#[test]
fn middleware_can_short_circuit() {
    let mut router = Router::new();
    router.get("/hi", hello);

    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);

    let (handle, join) = Server::new(router)
        .with_middleware(ShortCircuit)
        .spawn(&addr.to_string())
        .unwrap();
    thread::sleep(std::time::Duration::from_millis(50));

    let res = reqwest::blocking::get(format!("http://{}/hi", addr)).unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.text().unwrap(), "short");

    handle.shutdown();
    join.join().unwrap();
}

#[test]
fn middleware_can_reject() {
    let mut router = Router::new();
    router.get("/hi", hello);

    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);

    let (handle, join) = Server::new(router)
        .with_middleware(Reject)
        .spawn(&addr.to_string())
        .unwrap();
    thread::sleep(std::time::Duration::from_millis(50));

    let res = reqwest::blocking::get(format!("http://{}/hi", addr)).unwrap();
    assert_eq!(res.status(), 403);

    handle.shutdown();
    join.join().unwrap();
}
