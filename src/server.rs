use crate::middleware::Middleware;
use crate::request::Request;
use crate::response::{Body, Response};
use crate::router::Router;
use crate::status::Status;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

pub struct Server {
    pub router: Arc<Router>,
    pub middleware: Vec<Arc<dyn Middleware>>,
}

pub struct ServerHandle {
    shutdown: Arc<AtomicBool>,
}

impl ServerHandle {
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

impl Server {
    pub fn new(router: Router) -> Self {
        Self {
            router: Arc::new(router),
            middleware: Vec::new(),
        }
    }

    /// Attach a middleware to run for every request, in registration order.
    pub fn with_middleware<M: Middleware>(mut self, mw: M) -> Self {
        self.middleware.push(Arc::new(mw));
        self
    }

    pub fn run_blocking(self, addr: &str) -> std::io::Result<()> {
        let (_handle, join) = self.spawn(addr)?;
        join.join().expect("accept loop panicked");
        Ok(())
    }

    pub fn spawn(self, addr: &str) -> std::io::Result<(ServerHandle, JoinHandle<()>)> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown);
        let router = self.router;
        let middleware = Arc::new(self.middleware);

        let join = std::thread::Builder::new()
            .name("fornthulani-accept".into())
            .spawn(move || {
                while !shutdown_clone.load(Ordering::SeqCst) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let router = Arc::clone(&router);
                            let middleware = Arc::clone(&middleware);
                            std::thread::spawn(move || handle_conn(stream, router, middleware));
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(10));
                        }
                        Err(_) => break,
                    }
                }
            })?;

        Ok((ServerHandle { shutdown }, join))
    }
}

fn handle_conn(
    mut stream: TcpStream,
    router: Arc<Router>,
    middleware: Arc<Vec<Arc<dyn Middleware>>>,
) {
    let mut req = match Request::from_stream(&stream) {
        Ok(r) => r,
        Err(_) => {
            let _ = Response::text(Status::BAD_REQUEST, "Bad Request").write_to(&mut stream);
            return;
        }
    };

    // Global before — can reject or short-circuit.
    for mw in middleware.iter() {
        match mw.before(&mut req) {
            Ok(None) => continue,
            Ok(Some(mut resp)) => {
                // Short-circuit: run remaining `after` hooks on this response.
                for mw2 in middleware.iter() {
                    mw2.after(&req, &mut resp);
                }
                let _ = resp.write_to(&mut stream);
                return;
            }
            Err(e) => {
                let _ = e.into_response().write_to(&mut stream);
                return;
            }
        }
    }

    // Route dispatch (route-level middleware runs inside).
    let body: Body = router.dispatch(&req);

    // Global after — buffered responses only.
    let body = match body {
        Body::Buffered(mut resp) => {
            // Reverse order so `after` hooks nest properly.
            for mw in middleware.iter().rev() {
                mw.after(&req, &mut resp);
            }
            Body::Buffered(resp)
        }
        other => other,
    };

    let _ = body.write_to(&mut stream);
}
