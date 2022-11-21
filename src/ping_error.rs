use std::{error::Error, fmt};

pub type GenericError = Box<dyn Error + Send + Sync + 'static>;

// TODO: reuse standard errors whenever the semantics line up.
#[derive(Debug)]
pub struct PingError {
    pub message: String,
    // no chained error
}

impl fmt::Display for PingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "PingError")?;
        if !self.message.is_empty() {
            write!(f, ": {}", self.message)?;
        }
        Ok(())
    }
}

impl Error for PingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

// TODO: do we need these From<> implementations?
impl From<std::io::Error> for PingError {
    fn from(error: std::io::Error) -> PingError {
        PingError {
            message: error.to_string(),
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
        assert!(ping_error.source().is_none());
    }
}
