use crate::error::ThulaniError;
use crate::request::Request;
use crate::response::Response;

/// A hook into the request/response lifecycle.
///
/// - `before` runs after the request is parsed and before the controller.
///   Returning `Ok(None)` continues dispatch.
///   Returning `Ok(Some(resp))` **short-circuits**: the controller is
///   skipped and `resp` is sent to the client directly.
///   Returning `Err(e)` rejects the request; `e` becomes the response.
///
/// - `after` runs after dispatch, in reverse registration order.
///   It sees the request and can mutate the response (add headers,
///   set cookies, etc.).
///
/// Both methods have defaults, so a middleware only implements what
/// it needs.
pub trait Middleware: Send + Sync + 'static {
    fn before(&self, _req: &mut Request) -> Result<Option<Response>, ThulaniError> {
        Ok(None)
    }

    fn after(&self, _req: &Request, _resp: &mut Response) {}
}
