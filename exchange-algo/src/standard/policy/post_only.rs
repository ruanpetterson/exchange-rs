use exchange_core::{Asset, Exchange, Opposite as _, Trade};

use super::seq;
use super::Policy;

pub(super) struct PostOnly;
impl<O, E> Policy<O, E, seq::Before> for PostOnly
where
    E: Exchange,
    <E as Exchange>::Order: Trade<O>,
    O: Asset<
        OrderAmount = <<E as Exchange>::Order as Asset>::OrderAmount,
        OrderId = <<E as Exchange>::Order as Asset>::OrderId,
        OrderPrice = <<E as Exchange>::Order as Asset>::OrderPrice,
        OrderSide = <<E as Exchange>::Order as Asset>::OrderSide,
        OrderStatus = <<E as Exchange>::Order as Asset>::OrderStatus,
    >,
{
    #[inline]
    fn enforce(incoming_order: &mut O, exchange: &E) {
        if incoming_order.is_post_only()
            && exchange
                .peek(&incoming_order.side().opposite())
                .is_some_and(|top_order| {
                    top_order.matches(incoming_order).is_ok()
                })
        {
            // Post-only orders must go directly to orderbook and do not be
            // executed as taker at all, otherwise it must be cancelled before
            // enter the book.
            incoming_order.cancel();
        }
    }
}
