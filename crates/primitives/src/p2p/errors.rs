use strum::IntoStaticStr;

#[derive(Debug, Clone, Copy, PartialEq, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum ErrorCode {
    RateLimited,
    InvalidRequest,
    ServerError,
    Unknown,
}

impl std::error::Error for ErrorCode {}

impl ErrorCode {
    pub fn as_u8(&self) -> u8 {
        match self {
            ErrorCode::RateLimited => 139,
            ErrorCode::InvalidRequest => 1,
            ErrorCode::ServerError => 2,
            ErrorCode::Unknown => 255,
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            ErrorCode::InvalidRequest => "The request was invalid",
            ErrorCode::ServerError => "Server error occurred",
            ErrorCode::Unknown => "Unknown error occurred",
            ErrorCode::RateLimited => "Rate limited",
        };
        f.write_str(repr)
    }
}
