use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Unhandled(String),

    #[error("HttpRequest: {0}")]
    HttpRequest(reqwest::Error),

    #[error("YouTube Request: message: {message} endpoint: {endpoint} request data: {request_data:?}")]
    YtRequest {
        message:  String,
        endpoint: String,
        request_data:     serde_json::Value,
    },

    #[error("YouTube returned JSON that couldn't be parsed: {0}")]
    JsonParse(String),
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self { Self::HttpRequest(value) }
}
