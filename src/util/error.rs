use std::os::raw::c_int;

type ErrorNum = c_int;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub error_num: ErrorNum,
}

impl Error {
    pub fn new(message: &str, error_num: Option<ErrorNum>) -> Error {
        Error {
            message: message.to_string(),
            error_num: error_num.unwrap_or(libc::EOPNOTSUPP),
        }
    }
}
