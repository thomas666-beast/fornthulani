use crate::error::ThulaniError;
use crate::request::Request;
use crate::response::{Body, Response};
use crate::router::Params;

/// A controller is any function with this signature.
/// Register it directly with `Router::get` / `Router::post`.
pub type Controller = fn(&Request, &Params) -> Result<Response, ThulaniError>;

/// Same as [`Controller`] but returns a streaming body.
pub type StreamController = fn(&Request, &Params) -> Result<Body, ThulaniError>;
