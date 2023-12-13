use std::ops::ControlFlow;

use exchange_core::{Asset, Exchange, Opposite};
use num::Zero;

use super::Policy;

pub(super) struct FillOrKill;
impl<E: Exchange> Policy<E> for FillOrKill {
    #[inline]
    fn enforce(incoming_order: &mut E::Order, exchange: &E) {
        if incoming_order.is_fill_or_kill()
            && !FillOrKill::can_fill(incoming_order, exchange)
        {
            // The exchange should possess a sufficient number of orders to
            // execute an all-or-none order; otherwise, the all-or-none
            // order must be cancelled.
            incoming_order.cancel();
        }
    }
}

impl FillOrKill {
    /// Returns if `incoming_order` can be completely filled within given
    /// exchange.
    ///
    /// `incoming_order` is the order you want to check if it can be filled,
    /// and `exchange` is the orderbook that we'll use to compare against
    /// the given order.
    ///
    /// `can_fill()` is short-circuiting; in other words, it will stop
    /// processing as soon as it ensures the given order can be full-filled,
    /// given that no matter what else happens, the result will also be
    /// `true`.
    #[inline]
    fn can_fill<E: Exchange>(incoming_order: &E::Order, exchange: &E) -> bool {
        exchange
            .iter(&incoming_order.side().opposite())
            .take_while(|order| {
                // Gather only the orders that are compatible to the
                // `incoming_order`.
                order.matches(incoming_order).is_ok()
            })
            .map(|order| order.remaining())
            .try_fold(
                incoming_order.remaining(),
                |mut remaining, available_to_trade| {
                    remaining = remaining - available_to_trade.min(remaining);

                    // This means that the `incoming_order` can be fully
                    // filled.
                    if remaining.is_zero() {
                        // Using `ControlFlow` make this call short-circuiting;
                        // in other words, it will stop processing as soon as
                        // the closure returns `ControlFlow::Break`.
                        return ControlFlow::Break(remaining);
                    }

                    ControlFlow::Continue(remaining)
                },
            )
            .is_break()
    }
}
