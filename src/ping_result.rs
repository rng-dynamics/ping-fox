use crate::GenericError;

pub type PingResult<T> = std::result::Result<T, GenericError>;
