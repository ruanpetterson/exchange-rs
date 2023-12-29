use exchange_core::Asset;
use thiserror::Error;

use super::{Order, OrderId};

#[derive(Debug)]
#[cfg_attr(test, derive(Copy, Clone))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trade {
    pub(crate) taker: OrderId,
    pub(crate) maker: OrderId,
    pub(crate) amount: u64,
    pub(crate) price: u64,
}

impl Trade {
    /// Constructs a new `Trade`, returning an error if something fails.
    #[inline]
    pub fn new(
        taker: &mut Order,
        maker: &mut Order,
    ) -> Result<Trade, TradeError> {
        taker.matches(&*maker)?;

        let exchanged = taker.remaining().min(maker.remaining());
        let price =
            maker.limit_price().expect("maker must always have a price");

        taker.fill(exchanged);
        maker.fill(exchanged);

        Ok(Trade {
            taker: taker.id(),
            maker: maker.id(),
            amount: exchanged,
            price,
        })
    }

    /// Returns the traded price.
    #[inline]
    pub fn price(&self) -> u64 {
        self.price
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
