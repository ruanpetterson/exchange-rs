use exchange_core::{Asset, Exchange, Opposite};

use super::Policy;

pub(super) struct PostOnly;
impl<E: Exchange> Policy<E> for PostOnly {
    #[inline]
    fn enforce(incoming_order: &mut E::Order, exchange: &E) {
        if incoming_order.is_post_only()
            && exchange
                .peek(&incoming_order.side().opposite())
                .is_some_and(|top_order| {
                    incoming_order.matches(top_order).is_ok()
                })
        {
            // Post-only orders must go directly to orderbook and do not be
            // executed as taker at all, otherwise it must be cancelled before
            // enter the book.
            incoming_order.cancel();
        }
    }
}
