use orderbook_core::Asset;
use thiserror::Error;

use super::{Order, OrderId};

#[derive(Debug, Error)]
pub enum TradeError {
    #[error(transparent)]
    PriceError(#[from] PriceError),
    #[error(transparent)]
    SideError(#[from] SideError),
}

#[derive(Debug, Error)]
pub enum SideError {
    #[error("taker and maker must be at opposite sides")]
    Mismatch,
}

#[derive(Debug, Error)]
pub enum PriceError {
    #[error("prices do not match each other")]
    Mismatch,
}

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
    pub fn new(
        taker: &mut Order,
        maker: &mut Order,
    ) -> Result<Trade, TradeError> {
        if !taker.matches(maker) {
            return Err(TradeError::PriceError(PriceError::Mismatch));
        }

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
}
