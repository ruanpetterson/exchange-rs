#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "UPPERCASE"))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum OrderType {
    Limit {
        limit_price: u64,
        /// A conditional limit order that serves to add liquidity to the order
        /// book.
        #[cfg_attr(feature = "serde", serde(default))]
        post_only: bool,
    },
    Market,
}
