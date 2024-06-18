use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("unable to convert a non-limit order into limit")]
    Incompatible,
}

#[derive(Debug, Error)]
pub enum OrderError {
    #[error(transparent)]
    Conversion(#[from] ConversionError),
    #[error("empty filling is not allowed")]
    NoFill,
    #[error("filling quantity exceeds available quantity")]
    Overfill,
}

#[derive(Debug, Error)]
pub enum TradeError {
    #[error(transparent)]
    Price(#[from] PriceError),
    #[error("incompatible side")]
    SameSide,
    #[error(transparent)]
    Status(#[from] StatusError),
}

#[derive(Debug, Error)]
pub enum PriceError {
    #[error("incompatible price")]
    Incompatible,
    #[error("limit price not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum StatusError {
    #[error("order closed")]
    Closed,
}
