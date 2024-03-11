use exchange_core::{Asset as _, Exchange, Opposite as _, Trade as _};

use super::Policy;

pub(super) struct PostOnly;
impl<E: Exchange> Policy<E> for PostOnly {
    #[inline]
    fn enforce(incoming_order: &mut E::IncomingOrder, exchange: &E) {
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
