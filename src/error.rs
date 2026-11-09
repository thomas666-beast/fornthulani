use crate::response::Response;
use crate::status::Status;

#[derive(Debug)]
pub enum ThulaniError {
    Io(std::io::Error),
    NotFound(String),
    MethodNotAllowed(String),
    BadRequest(String),
    Forbidden(String),
    Internal(String),
}

impl std::fmt::Display for ThulaniError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThulaniError::Io(e) => write!(f, "io error: {e}"),
            ThulaniError::NotFound(s) => write!(f, "not found: {s}"),
            ThulaniError::MethodNotAllowed(s) => write!(f, "method not allowed: {s}"),
            ThulaniError::BadRequest(s) => write!(f, "bad request: {s}"),
            ThulaniError::Forbidden(s) => write!(f, "forbidden: {s}"),
            ThulaniError::Internal(s) => write!(f, "internal error: {s}"),
        }
    }
}

impl std::error::Error for ThulaniError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ThulaniError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ThulaniError {
    fn from(e: std::io::Error) -> Self {
        ThulaniError::Io(e)
    }
}

impl ThulaniError {
    pub fn status(&self) -> Status {
        match self {
            ThulaniError::NotFound(_) => Status::NOT_FOUND,
            ThulaniError::MethodNotAllowed(_) => Status::METHOD_NOT_ALLOWED,
            ThulaniError::BadRequest(_) => Status::BAD_REQUEST,
            ThulaniError::Forbidden(_) => Status::FORBIDDEN,
            ThulaniError::Io(_) | ThulaniError::Internal(_) => Status::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn into_response(self) -> Response {
        let status = self.status();
        let msg = self.to_string();
        Response::text(status, &msg)
    }
}
