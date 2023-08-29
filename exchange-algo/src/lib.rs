use std::ops::{Deref, DerefMut};

use exchange_core::Policy;
use exchange_core::{Asset, Exchange, ExchangeExt, Opposite};
use policy::*;
use thiserror::Error;

pub struct DefaultExchange<E>(E);

impl<E> Deref for DefaultExchange<E> {
    type Target = E;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for DefaultExchange<E> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E> From<E> for DefaultExchange<E>
where
    E: Exchange + ExchangeExt,
{
    #[inline]
    fn from(e: E) -> Self {
        Self(e)
    }
}

impl<E> DefaultExchange<E>
where
    E: Exchange + ExchangeExt,
{
    /// Core exchange algorithm.
    #[inline]
    pub fn matching(
        &mut self,
        order: <E as Exchange>::Order,
    ) -> Result<(), DefaultExchangeError> {
        let mut incoming_order = order;

        // Define order policies to be run before matching and apply them.
        let before_policies: &[&dyn Policy<E::Order, E>] =
            &[&AllOrNone, &PostOnly];

        before_policies
            .iter()
            .for_each(|policy| policy.enforce(&mut incoming_order, self));

        while let (false, Some(top_order)) = (
            incoming_order.is_closed(),
            self.peek_mut(&incoming_order.side().opposite()),
        ) {
            let Some(_trade) = incoming_order.trade(top_order) else {
                // Since incoming order is not matching to top order anymore, we
                // can move on.
                break;
            };

            if top_order.is_closed() {
                // As long as top order is completed, it can be safely removed
                // from orderbook.
                self.pop(&incoming_order.side().opposite())
                    .expect("top order should be `Some`");
            }
        }

        // Define order policies to be run after matching and apply them.
        let late_policies: &[&dyn Policy<E::Order, E>] = &[&ImmediateOrCancel];

        late_policies
            .iter()
            .for_each(|policy| policy.enforce(&mut incoming_order, self));

        // If incoming order is not full-filled and open, it must be inserted
        // into the orderbook.
        if !incoming_order.is_closed() {
            self.insert(incoming_order);
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum DefaultExchangeError {}

mod policy {
    use super::*;

    pub(super) struct AllOrNone;
    impl<E: Exchange + ExchangeExt> Policy<E::Order, E> for AllOrNone {
        #[inline]
        fn enforce(&self, incoming_order: &mut E::Order, exchange: &E) {
            if incoming_order.is_all_or_none()
                && incoming_order.remaining()
                    > exchange.volume_with(
                        incoming_order.side().opposite(),
                        |order| order.matches(&incoming_order),
                    )
            {
                // The exchange should possess a sufficient number of orders to
                // execute an all-or-none order; otherwise, the all-or-none
                // order must be cancelled.
                incoming_order.cancel();
            }
        }
    }

    pub(super) struct PostOnly;
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

    pub(super) struct ImmediateOrCancel;
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
