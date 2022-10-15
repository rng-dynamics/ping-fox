use std::{error::Error, fmt};

// If you want to derive Clone for PingError, you can, e.g., make GenericError = rc::Rc<...>
// instead of Box<...>.
pub type GenericError = Box<dyn Error + Send + Sync + 'static>;

// TODO: reuse standard errors whenever the semantics line up.
#[derive(Debug)]
pub struct PingError {
    pub message: String,
    // TODO: don't chain errors
    pub source: Option<GenericError>,
}

impl fmt::Display for PingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "PingError")?;
        if !self.message.is_empty() {
            write!(f, ": {}", self.message)?;
        }
        if let Some(e) = &self.source {
            write!(f, ": {}", e)?;
        }
        Ok(())
    }
}

impl Error for PingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        let e: Option<&(dyn Error + 'static)> = match &self.source {
            None => None,
            Some(s) => Some(&**s),
        };
        e
    }
}

// TODO: do we need these From<> implementations?
impl From<std::io::Error> for PingError {
    fn from(error: std::io::Error) -> PingError {
        PingError {
            message: "".to_owned(),
            source: Some(Box::new(error)),
        }
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for PingError
where
    T: Send + Sync + 'static,
{
    fn from(error: std::sync::mpsc::SendError<T>) -> PingError {
        PingError {
            message: "".to_owned(),
            source: Some(Box::new(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use super::*;

    #[test]
    fn test_ping_error_from_std_io_error() {
        let std_io_error = std::io::Error::from(ErrorKind::Other);
        let ping_error: PingError = PingError::from(std_io_error);
        let source = ping_error.source().expect("missing source");
        assert_eq!("other error", source.to_string());
    }
}
