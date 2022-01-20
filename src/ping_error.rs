use std::{error::Error, fmt, rc::Rc};

type GenericError = Box<dyn Error + Send + Sync + 'static>;

// If you want to derive Clone for PingError, you can, e.g., make GenericError = Rc<...>
// instead of Box<...>.
#[derive(Debug)]
pub struct PingError {
    pub message: String,
    pub source: Option<GenericError>,
}
// TODO: test the error

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
        let e: Option<&(dyn Error + 'static)>;
        e = match &self.source {
            None => None,
            Some(s) => Some(&**s),
        };
        e
    }
}

impl From<std::io::Error> for PingError {
    fn from(error: std::io::Error) -> PingError {
        PingError {
            message: "".to_owned(),
            source: Some(Box::new(error)),
        }
    }
}
