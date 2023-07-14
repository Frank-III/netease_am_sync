use thiserror::Error;

#[derive(Error, Debug)]
pub enum NeteaseCallError {
    #[error("Client Error: Fail to connect to netease client with {0:?}")]
    ClientFailError(i64),
    #[error("Parse Error: {0:?}")]
    ParseError(String),
    #[error("QR Code Error: {0:?}")]
    QrCodeError(String),
    #[error("You should login first")]
    NoCookieError,
}

pub(crate) type NetResult<T> = Result<T, NeteaseCallError>;

// #[derive(Debug)]
// pub struct ParseError(&'static str);
//
// impl ParseError {
//     pub fn with_message(message: &'static str) -> Self {
//         Self(message)
//     }
//
//     pub fn new() -> Self {
//         Self::with_message("parse failed.")
//     }
// }
//
// impl std::fmt::Display for ParseError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{}", self.0)
//     }
// }
//
// impl std::error::Error for ParseError {
//     fn description(&self) -> &str {
//         self.0
//     }
// }
