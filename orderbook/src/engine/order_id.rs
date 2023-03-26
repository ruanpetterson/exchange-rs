#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OrderId(u64);

impl OrderId {
    #[inline]
    pub fn new(order_id: u64) -> Self {
        Self(order_id)
    }
}
