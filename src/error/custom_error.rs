use std::{error::Error, fmt};

#[derive(Debug)]
pub struct CustomError {
    pub details: String,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for CustomError {}

impl CustomError {
    fn new(msg: &str) -> CustomError {
        CustomError {
            details: msg.to_string(),
        }
    }

    pub fn new_boxed(msg: &str) -> Box<dyn Error> {
        Box::new(CustomError::new(msg))
    }
}
