use orderbook_core::Asset;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::OrderId;

#[derive(Debug, Error)]
pub enum TradeError {
    #[error("taker and maker must be opposite each other")]
    MismatchSides,
}

#[derive(Debug)]
#[cfg_attr(test, derive(Copy, Clone))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Trade {
    pub(crate) taker: OrderId,
    pub(crate) maker: OrderId,
    pub(crate) amount: u64,
    pub(crate) price: u64,
}

impl<Order: Asset<Trade = Self>> TryFrom<(&mut Order, &mut Order)> for Trade {
    type Error = TradeError;

    #[inline]
    fn try_from(
        (taker, maker): (&mut Order, &mut Order),
    ) -> Result<Self, Self::Error> {
        taker.trade(maker).ok_or(TradeError::MismatchSides)
    }
}
