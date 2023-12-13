use crate::{Exchange, ExchangeExt};

/// Core exchange algorithm.
pub trait Algo {
    type Error;
    type Output;

    /// Attempt to match an incoming order.
    ///
    /// This method takes an order as input and attempts to match it against the
    /// existing limit orders in the orderbook. Matching is done in a specific
    /// order based on the orderbook's rules, such as price-time priority.
    fn matching<E>(
        exchange: &mut E,
        incoming_order: <E as Exchange>::Order,
    ) -> Result<Self::Output, Self::Error>
    where
        E: Exchange + ExchangeExt;
}
