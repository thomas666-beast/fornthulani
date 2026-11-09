#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status(pub u16);

impl Status {
    pub const OK: Status = Status(200);
    pub const CREATED: Status = Status(201);
    pub const NO_CONTENT: Status = Status(204);
    pub const BAD_REQUEST: Status = Status(400);
    pub const UNAUTHORIZED: Status = Status(401);
    pub const FORBIDDEN: Status = Status(403);
    pub const NOT_FOUND: Status = Status(404);
    pub const METHOD_NOT_ALLOWED: Status = Status(405);
    pub const CONFLICT: Status = Status(409);
    pub const UNPROCESSABLE_ENTITY: Status = Status(422);
    pub const INTERNAL_SERVER_ERROR: Status = Status(500);

    pub fn code(self) -> u16 {
        self.0
    }

    pub fn reason(self) -> &'static str {
        match self.0 {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            409 => "Conflict",
            422 => "Unprocessable Entity",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.reason())
    }
}
