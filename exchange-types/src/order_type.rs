use rust_decimal::Decimal;

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
        limit_price: Decimal,
        /// Time in force policies provide guarantees about the lifetime of an
        /// [order](Order).
        #[cfg_attr(feature = "serde", serde(default))]
        time_in_force: TimeInForce,
        amount: Decimal,
        #[cfg_attr(feature = "serde", serde(default))]
        filled: Decimal,
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
        amount: Decimal,
        #[cfg_attr(feature = "serde", serde(default))]
        filled: Decimal,
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
    GoodTillCancel {
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
        Self::GoodTillCancel { post_only: false }
    }
}

#[cfg(feature = "rand")]
mod __rand {
    use rand::distributions::Standard;
    use rand::prelude::*;

    use super::*;

    impl Distribution<OrderType> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> OrderType {
            match rng.gen_bool(0.8) {
                true => OrderType::Limit {
                    limit_price: rng.gen_range(
                        Decimal::new(100, 0)..Decimal::new(10_000, 0),
                    ),
                    time_in_force: rng.gen(),
                    amount: rng.gen_range(
                        Decimal::new(100, 0)..Decimal::new(10_000, 0),
                    ),
                    filled: Decimal::ZERO,
                },
                false => OrderType::Market {
                    all_or_none: rng.gen_bool(0.01),
                    amount: rng.gen_range(
                        Decimal::new(100, 0)..Decimal::new(10_000, 0),
                    ),
                    filled: Decimal::ZERO,
                },
            }
        }
    }

    impl Distribution<TimeInForce> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TimeInForce {
            match rng.gen_bool(0.95) {
                true => TimeInForce::GoodTillCancel {
                    post_only: rng.gen_bool(0.1),
                },
                false => TimeInForce::ImmediateOrCancel {
                    all_or_none: rng.gen_bool(0.05),
                },
            }
        }
    }
}
