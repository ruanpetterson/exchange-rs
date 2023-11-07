use exchange_core::Opposite;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "UPPERCASE"))]
pub enum Side {
    #[cfg_attr(feature = "serde", serde(alias = "SELL"))]
    Ask,
    #[cfg_attr(feature = "serde", serde(alias = "BUY"))]
    Bid,
}

impl Opposite for Side {
    #[inline]
    fn opposite(&self) -> Self {
        match self {
            Side::Ask => Side::Bid,
            Side::Bid => Side::Ask,
        }
    }
}
