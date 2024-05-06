use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, strum_macros::AsRefStr)]
pub enum Error {
    LoginFail,

    // Model errors.
    LinkCreateFailedBadProtocol,
    LinkCreateFailedDBError,
    LinkCreateFailedEmptyTarget,
    LinkCreateFailedTargetTooLong,
    LinkCreateFailedTooManyCollisions,
    LinkGetDBError,
    LinkGetNoSuchLink,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");

        // Start with the default fallback error.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert a more specific error if we know one.
        response.extensions_mut().insert(self);

        response
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError, &str) {
        #[allow(unreachable_patterns)]
        match self {
            // Model errors.
            Self::LinkCreateFailedBadProtocol => (
                StatusCode::BAD_REQUEST,
                ClientError::INVALID_PARAMS,
                "target must have http or https protocol",
            ),

            Self::LinkCreateFailedDBError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
                "Database error", // Deliberately vague since it's being shown to the user.
            ),

            Self::LinkCreateFailedEmptyTarget => (
                StatusCode::BAD_REQUEST,
                ClientError::INVALID_PARAMS,
                "target must not be empty",
            ),

            Self::LinkCreateFailedTargetTooLong => (
                StatusCode::PAYLOAD_TOO_LARGE,
                ClientError::INVALID_PARAMS,
                "target is too long",
            ),

            Self::LinkGetDBError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
                "Database error", // Deliberately vague since it's being shown to the user.
            ),

            Self::LinkGetNoSuchLink => (
                StatusCode::NOT_FOUND,
                ClientError::LINK_NOT_FOUND,
                "Link not found",
            ),

            // Fallback when no other error is appropriate.
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
                "Service error",
            ),
        }
    }
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LINK_NOT_FOUND,
    LOGIN_FAIL,
    NO_AUTH,
    INVALID_PARAMS,
    SERVICE_ERROR,
}
