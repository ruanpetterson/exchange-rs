use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("unable to convert a non-limit order into limit")]
    Incompatible,
    #[error(transparent)]
    Status(#[from] StatusError),
}

#[derive(Debug, Error)]
pub enum OrderError {
    #[error(transparent)]
    Conversion(#[from] ConversionError),
    #[error("empty filling is not allowed")]
    NoFill,
    #[error("filling amount exceeds remaining amount")]
    Overfill,
}

impl From<StatusError> for OrderError {
    #[inline]
    fn from(error: StatusError) -> Self {
        ConversionError::from(error).into()
    }
}

#[derive(Debug, Error)]
pub enum TradeError {
    #[error(transparent)]
    Price(#[from] PriceError),
    #[error(transparent)]
    Side(#[from] SideError),
    #[error(transparent)]
    Status(#[from] StatusError),
}

#[derive(Debug, Error)]
pub enum PriceError {
    #[error("prices do not match each other")]
    Incompatible,
    #[error("limit price is a must")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum SideError {
    #[error("taker and maker must be at opposite sides")]
    Conflict,
}

#[derive(Debug, Error)]
pub enum StatusError {
    #[error("taker and maker cannot be closed")]
    Closed,
}
