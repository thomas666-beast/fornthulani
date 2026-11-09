use crate::controller::Controller;
use crate::error::ThulaniError;
use crate::middleware::Middleware;
use crate::request::Request;
use crate::response::{Body, Response};
use crate::route::Route;
use crate::status::Status;
use std::collections::HashMap;
use std::sync::Arc;

/// Route parameters extracted from the URL.
pub struct Params(HashMap<String, String>);

impl Params {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.get(name).map(String::as_str)
    }

    pub fn require(&self, name: &str) -> Result<&str, ThulaniError> {
        self.0
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| ThulaniError::Internal(format!("missing route param `{name}`")))
    }
}

#[derive(Clone)]
pub(crate) enum Segment {
    Literal(String),
    Param(String),
}

pub(crate) struct RegisteredRoute {
    pub(crate) method: String,
    pub(crate) pattern: Vec<Segment>,
    pub(crate) handler: Controller,
    pub(crate) name: Option<String>,
    pub(crate) middleware: Vec<Arc<dyn Middleware>>,
}

#[derive(Default)]
pub struct Router {
    routes: Vec<RegisteredRoute>,
    prefix_stack: Vec<String>,
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }

    // ---------- route registration ----------

    pub fn get(&mut self, path: &str, handler: Controller) -> Route<'_> {
        self.register("GET", path, handler)
    }

    pub fn post(&mut self, path: &str, handler: Controller) -> Route<'_> {
        self.register("POST", path, handler)
    }

    pub fn put(&mut self, path: &str, handler: Controller) -> Route<'_> {
        self.register("PUT", path, handler)
    }

    pub fn delete(&mut self, path: &str, handler: Controller) -> Route<'_> {
        self.register("DELETE", path, handler)
    }

    /// Register a set of routes under a shared prefix.
    ///
    /// ```ignore
    /// router.group("/api/v1", |r| {
    ///     r.get("/users", list_users);
    ///     r.post("/users", create_user);
    /// });
    /// ```
    pub fn group<F>(&mut self, prefix: &str, f: F) -> &mut Self
    where
        F: FnOnce(&mut Router),
    {
        self.prefix_stack.push(normalize_prefix(prefix));
        f(self);
        self.prefix_stack.pop();
        self
    }

    // ---------- internal ----------

    fn register(&mut self, method: &str, path: &str, handler: Controller) -> Route<'_> {
        let full_path = self.compose_path(path);
        let pattern = parse_pattern(&full_path);
        let index = self.routes.len();
        self.routes.push(RegisteredRoute {
            method: method.to_ascii_uppercase(),
            pattern,
            handler,
            name: None,
            middleware: Vec::new(),
        });
        Route {
            router: self,
            index,
        }
    }

    fn compose_path(&self, path: &str) -> String {
        if self.prefix_stack.is_empty() {
            return path.to_string();
        }
        let mut out = String::new();
        for p in &self.prefix_stack {
            out.push_str(p);
        }
        out.push_str(path);
        out
    }

    pub(crate) fn set_route_name(&mut self, index: usize, name: &str) {
        if let Some(r) = self.routes.get_mut(index) {
            r.name = Some(name.to_string());
        }
    }

    pub(crate) fn add_route_middleware(&mut self, index: usize, mw: Arc<dyn Middleware>) {
        if let Some(r) = self.routes.get_mut(index) {
            r.middleware.push(mw);
        }
    }

    /// Returns the path pattern registered under `name`, if any.
    pub fn path_for(&self, name: &str) -> Option<String> {
        self.routes
            .iter()
            .find(|r| r.name.as_deref() == Some(name))
            .map(|r| pattern_to_path(&r.pattern))
    }

    // ---------- dispatch ----------

    pub fn dispatch(&self, req: &Request) -> Body {
        let parts = split_path(&req.path);
        let method = req.method.to_ascii_uppercase();
        let mut method_mismatch = false;

        for route in &self.routes {
            if route.pattern.len() != parts.len() {
                continue;
            }

            let mut params = HashMap::new();
            let mut path_ok = true;
            for (seg, part) in route.pattern.iter().zip(&parts) {
                match seg {
                    Segment::Literal(lit) if !lit.eq_ignore_ascii_case(part) => {
                        path_ok = false;
                        break;
                    }
                    Segment::Literal(_) => {}
                    Segment::Param(name) => {
                        params.insert(name.clone(), (*part).to_string());
                    }
                }
            }
            if !path_ok {
                continue;
            }

            if route.method != method {
                method_mismatch = true;
                continue;
            }

            // Route-level middleware runs before + after the controller.
            let mut owned_req = Request {
                method: req.method.clone(),
                path: req.path.clone(),
                headers: req.headers.clone(),
                body: req.body.clone(),
            };

            for mw in &route.middleware {
                match mw.before(&mut owned_req) {
                    Ok(None) => continue,
                    Ok(Some(mut resp)) => {
                        for mw2 in route.middleware.iter().rev() {
                            mw2.after(&owned_req, &mut resp);
                        }
                        return resp.into();
                    }
                    Err(e) => return e.into_response().into(),
                }
            }

            let params = Params(params);
            let mut resp = match (route.handler)(&owned_req, &params) {
                Ok(r) => r,
                Err(e) => e.into_response(),
            };

            for mw in route.middleware.iter().rev() {
                mw.after(&owned_req, &mut resp);
            }

            return resp.into();
        }

        let body = if method_mismatch {
            Response::text(
                Status::METHOD_NOT_ALLOWED,
                &format!("method {method} not allowed on {}", req.path),
            )
        } else {
            Response::text(Status::NOT_FOUND, "Not Found")
        };
        body.into()
    }
}

fn normalize_prefix(prefix: &str) -> String {
    let trimmed = prefix.trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("/{trimmed}")
    }
}

fn parse_pattern(path: &str) -> Vec<Segment> {
    split_path(path)
        .into_iter()
        .map(|s| {
            if let Some(name) = s.strip_prefix(':') {
                Segment::Param(name.to_string())
            } else {
                Segment::Literal(s.to_string())
            }
        })
        .collect()
}

fn pattern_to_path(pattern: &[Segment]) -> String {
    let mut out = String::new();
    for seg in pattern {
        out.push('/');
        match seg {
            Segment::Literal(l) => out.push_str(l),
            Segment::Param(p) => {
                out.push(':');
                out.push_str(p);
            }
        }
    }
    if out.is_empty() {
        out.push('/');
    }
    out
}

fn split_path(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty()).collect()
}
