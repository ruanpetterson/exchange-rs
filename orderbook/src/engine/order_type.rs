#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "UPPERCASE"))]
pub enum OrderType {
    Limit {
        /// A conditional limit order that serves to add liquidity to the order
        /// book.
        #[cfg_attr(feature = "serde", serde(default))]
        post_only: bool,
    },
    Market,
}

impl Default for OrderType {
    fn default() -> Self {
        Self::Limit { post_only: false }
    }
}
