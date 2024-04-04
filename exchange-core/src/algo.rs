use crate::Asset;
use crate::Exchange;
use crate::ExchangeExt;
use crate::Trade;

/// Core exchange algorithm.
pub trait Algo<O> {
    type Error;
    type Output;

    /// Attempt to match an incoming order.
    ///
    /// This method takes an order as input and attempts to match it against the
    /// existing limit orders in the orderbook. Matching is done in a specific
    /// order based on the orderbook's rules, such as price-time priority.
    fn matching<E>(
        exchange: &mut E,
        incoming_order: O,
    ) -> Result<Self::Output, Self::Error>
    where
        E: Exchange + ExchangeExt,
        <E as Exchange>::Order: Trade<O> + TryFrom<O>,
        O: Asset<
            OrderAmount = <<E as Exchange>::Order as Asset>::OrderAmount,
            OrderId = <<E as Exchange>::Order as Asset>::OrderId,
            OrderPrice = <<E as Exchange>::Order as Asset>::OrderPrice,
            OrderSide = <<E as Exchange>::Order as Asset>::OrderSide,
            OrderStatus = <<E as Exchange>::Order as Asset>::OrderStatus,
        >;
}
