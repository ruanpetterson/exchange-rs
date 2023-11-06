#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Id(u64);

impl Id {
    #[inline]
    pub fn new(order_id: u64) -> Self {
        Self(order_id)
    }
}
