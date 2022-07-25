use std::{error::Error, fmt};

#[derive(Debug)] // Allow the use of "{:?}" format specifier
pub enum Ip2RegionError {
    NoneError(String),
}

// Allow the use of "{}" format specifier
impl fmt::Display for Ip2RegionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ip2RegionError::NoneError(msg) => write!(f, "{}", msg),
        }
    }
}
impl Error for Ip2RegionError {}
