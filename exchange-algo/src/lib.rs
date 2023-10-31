use exchange_core::{Algo, Policy};
use exchange_core::{Asset, Exchange, ExchangeExt, Opposite};
use num::Zero;
use thiserror::Error;

pub struct DefaultExchange;

impl Algo for DefaultExchange {
    type Error = DefaultExchangeError;
    type Output = ();

    #[inline]
    fn matching<E: Exchange + ExchangeExt>(
        exchange: &mut E,
        order: <E as Exchange>::Order,
    ) -> Result<(), DefaultExchangeError> {
        let mut incoming_order = order;

        policy::before_policies()
            .iter()
            .for_each(|policy| policy.enforce(&mut incoming_order, exchange));

        while let (false, Some(top_order)) = (
            incoming_order.is_closed(),
            exchange.peek_mut(&incoming_order.side().opposite()),
        ) {
            let Some(_trade) = incoming_order.trade(top_order) else {
                // Since incoming order is not matching to top order anymore, we
                // can move on.
                break;
            };

            if top_order.is_closed() {
                // As long as top order is completed, it can be safely removed
                // from orderbook.
                exchange
                    .pop(&incoming_order.side().opposite())
                    .expect("top order should be `Some`");
            }
        }

        policy::late_policies()
            .iter()
            .for_each(|policy| policy.enforce(&mut incoming_order, exchange));

        // If incoming order is not full-filled and open, it must be inserted
        // into the orderbook.
        if !incoming_order.is_closed() {
            exchange.insert(incoming_order);
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DefaultExchangeError {}

mod policy {
    use super::*;

    /// Policies that should be run before matching.
    pub(super) const fn before_policies<'e, E: Exchange + ExchangeExt + 'e>(
    ) -> &'e [&'e dyn Policy<E::Order, E>] {
        &[&FillOrKill, &PostOnly]
    }

    /// Policies that should be run after matching.
    pub(super) const fn late_policies<'e, E: Exchange + ExchangeExt + 'e>(
    ) -> &'e [&'e dyn Policy<E::Order, E>] {
        &[&ImmediateOrCancel]
    }

    struct FillOrKill;
    impl<E: Exchange> Policy<E::Order, E> for FillOrKill {
        #[inline]
        fn enforce(&self, incoming_order: &mut E::Order, exchange: &E) {
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
        #[inline]
        fn can_fill<E: Exchange>(
            incoming_order: &E::Order,
            exchange: &E,
        ) -> bool {
            exchange
                .iter(&incoming_order.side().opposite())
                .take_while(|order| {
                    // Gather only the orders that are compatible to the
                    // `incoming_order`.
                    order.matches(incoming_order)
                })
                .map(|order| order.remaining())
                .reduce(|curr, acc| curr + acc)
                .unwrap_or(Zero::zero())
                >= incoming_order.remaining()
        }
    }

    struct PostOnly;
    impl<E: Exchange> Policy<E::Order, E> for PostOnly {
        #[inline]
        fn enforce(&self, incoming_order: &mut E::Order, exchange: &E) {
            if incoming_order.is_post_only()
                && !exchange
                    .peek(&incoming_order.side().opposite())
                    .is_some_and(|top_order| incoming_order.matches(top_order))
            {
                // Post-only orders must go directly to orderbook and do not be
                // executed as taker at all, otherwise it'll be canceled.
                incoming_order.cancel();
            }
        }
    }

    struct ImmediateOrCancel;
    impl<E: Exchange> Policy<E::Order, E> for ImmediateOrCancel {
        #[inline]
        fn enforce(&self, incoming_order: &mut E::Order, _: &E) {
            if incoming_order.is_immediate_or_cancel() {
                // If incoming order is immediate or cancel, it must be closed
                // at the end of matching.
                incoming_order.cancel();
            }
        }
    }
}
