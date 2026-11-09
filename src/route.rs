use crate::middleware::Middleware;
use std::sync::Arc;

/// Fluent builder returned by `Router::get` / `post` / `put` / `delete`.
///
/// ```ignore
/// router
///     .get("/users/:id", user_show)
///     .name("user_show")
///     .with(AuthMiddleware::new());
/// ```
pub struct Route<'a> {
    pub(crate) router: &'a mut crate::router::Router,
    pub(crate) index: usize,
}

impl<'a> Route<'a> {
    /// Attach a name to this route. Used by future URL builders.
    pub fn name(self, name: &str) -> Self {
        self.router.set_route_name(self.index, name);
        self
    }

    /// Attach a middleware to this route only.
    pub fn with<M: Middleware>(self, mw: M) -> Self {
        self.router.add_route_middleware(self.index, Arc::new(mw));
        self
    }
}
