#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Opposite;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "UPPERCASE"))]
pub enum OrderSide {
    #[cfg_attr(feature = "serde", serde(alias = "SELL"))]
    Ask,
    #[cfg_attr(feature = "serde", serde(alias = "BUY"))]
    Bid,
}

impl Opposite for OrderSide {
    #[inline]
    fn opposite(&self) -> Self {
        match self {
            OrderSide::Ask => OrderSide::Bid,
            OrderSide::Bid => OrderSide::Ask,
        }
    }
}
