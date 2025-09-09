use std::fmt::{Display, Formatter};

/// Universal error returned by Exlib
///
/// Includes source where possible
#[derive(Debug)]
pub struct Error {
    message: String,
    source: Option<Box<dyn std::error::Error>>,
}

impl Error {
    /// Creates a new instance of [Error]
    pub(crate) fn new(message: &str, source: Option<Box<dyn std::error::Error>>) -> Self {
        Self {
            message: String::from(message),
            source,
        }
    }
}

impl std::error::Error for Error {
    /// Returns the source error
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.source {
            Some(source) => return Some(source.as_ref()),
            None => return None,
        }
    }
}

impl Display for Error {
    /// Formats the error into human readable format
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(source) => return write!(f, "{}:{}", self.message, source),
            None => return write!(f, "{}", self.message),
        }
    }
}

/// Shorthand for [Error::new]
pub(crate) fn make_error(message: &str, source: Option<Box<dyn std::error::Error>>) -> Error {
    return Error::new(message, source);
}

/// Shorthand for ``Err(Error::new(...)``
pub(crate) fn make_error_result<T>(
    message: &str,
    source: Option<Box<dyn std::error::Error>>,
) -> Result<T, Error> {
    return Err(make_error(message, source));
}
