use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct GeneratorError {
    pub message: String,
}

impl Display for GeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl GeneratorError {
    pub(crate) fn new(message: String) -> GeneratorError {
        return GeneratorError {
            message,
        };
    }
}