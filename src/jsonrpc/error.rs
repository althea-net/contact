use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub enum JsonRpcError {
    BadResponse(String),
    FailedToSend(String),
    ResponseError {
        code: i64,
        message: String,
        data: String,
    },
}

impl Display for JsonRpcError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            JsonRpcError::BadResponse(val) => write!(f, "JsonRPC bad response {}", val),
            JsonRpcError::FailedToSend(val) => write!(f, "JsonRPC Failed to send {}", val),
            JsonRpcError::ResponseError {
                code,
                message,
                data,
            } => write!(
                f,
                "JsonRPC Response error code {} message {} data {:?}",
                code, message, data
            ),
        }
    }
}

impl Error for JsonRpcError {}
