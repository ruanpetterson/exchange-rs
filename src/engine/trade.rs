use crate::Asset;

use super::OrderId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TradeError {
    #[error("taker and maker must be opposite each other")]
    MismatchSides,
}

#[derive(Debug)]
#[cfg_attr(test, derive(Copy, Clone))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Trade {
    pub(super) taker: OrderId,
    pub(super) maker: OrderId,
    pub(super) amount: u64,
    pub(super) price: u64,
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
