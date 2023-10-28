/// The order type you specify will influence which other order parameters are
/// required as well as how your order will be executed by the matching engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "UPPERCASE"))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum OrderType {
    /// Limit orders are both the default and basic order type. A limit order
    /// requires specifying a price and size. The size is the number of bitcoin
    /// to buy or sell, and the price is the price per bitcoin. The limit order
    /// will be filled at the price specified or better.
    Limit {
        limit_price: u64,
        /// Time in force policies provide guarantees about the lifetime of an
        /// [order](Order).
        #[cfg_attr(feature = "serde", serde(default))]
        time_in_force: TimeInForce,
        amount: u64,
        #[cfg_attr(feature = "serde", serde(default))]
        filled: u64,
    },
    /// Market orders differ from limit orders in that they provide no pricing
    /// guarantees. They however do provide a way to buy or sell specific
    /// amounts of bitcoin or fiat without having to specify the price. Market
    /// orders execute immediately and no part of the market order will go on
    /// the open order book.
    Market {
        /// The `all or none` flag indicates that the orders are rejected if
        /// the entire size cannot be matched. When this is `true`, the order
        /// is considered a fill or kill order.
        #[cfg_attr(feature = "serde", serde(default))]
        all_or_none: bool,
        amount: u64,
        #[cfg_attr(feature = "serde", serde(default))]
        filled: u64,
    },
}

/// Time in force policies provide guarantees about the lifetime of an
/// [order](Order).
///
/// There are two policies: good till canceled
/// [`GTC`](TimeInForce::GoodTillCancel) and immediate or cancel
/// [`IOC`](TimeInForce::ImmediateOrCancel).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum TimeInForce {
    /// An order will be on the book unless the order is canceled.
    #[cfg_attr(feature = "serde", serde(rename = "GTC"))]
    GoodTilCancel {
        /// The post-only flag indicates that the order should only make
        /// liquidity. If any part of the order results in taking liquidity,
        /// the order will be rejected and no part of it will execute.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "core::ops::Not::not")
        )]
        post_only: bool,
    },
    /// An order will try to fill the order as much as it can before the order
    /// expires.
    #[cfg_attr(feature = "serde", serde(rename = "IOC"))]
    ImmediateOrCancel {
        /// The `all-or-none` flag indicates that the orders are rejected if
        /// the entire size cannot be matched. When this is `true`, the order
        /// is considered a fill or kill order.
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "core::ops::Not::not")
        )]
        all_or_none: bool,
    },
}

impl Default for TimeInForce {
    fn default() -> Self {
        Self::GoodTilCancel { post_only: false }
    }
}
