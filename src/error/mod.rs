use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Error {
    message: String,
    source: Option<Box<dyn std::error::Error>>,
}

impl Error {
    pub(crate) fn new(message: &str, source: Option<Box<dyn std::error::Error>>) -> Self {
        Self {
            message: String::from(message),
            source,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.source {
            Some(source) => return Some(source.as_ref()),
            None => return None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(source) => return write!(f, "{}:{}", self.message, source),
            None => return write!(f, "{}", self.message),
        }
    }
}

pub(crate) fn make_error(message: &str, source: Option<Box<dyn std::error::Error>>) -> Error {
    return Error::new(message, source);
}

pub(crate) fn make_error_result<T>(
    message: &str,
    source: Option<Box<dyn std::error::Error>>,
) -> Result<T, Error> {
    return Err(make_error(message, source));
}
