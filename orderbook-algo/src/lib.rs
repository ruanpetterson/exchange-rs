use std::ops::{Deref, DerefMut};

use orderbook_core::{Asset, Exchange, ExchangeExt, Opposite};

pub struct DefaultExchange<E>(E);

impl<E> Deref for DefaultExchange<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for DefaultExchange<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E> From<E> for DefaultExchange<E>
where
    E: Exchange + ExchangeExt,
{
    fn from(e: E) -> Self {
        Self(e)
    }
}

impl<E> DefaultExchange<E>
where
    E: Exchange + ExchangeExt,
{
    /// Core exchange algorithm.
    pub fn matching(
        &mut self,
        order: <E as Exchange>::Order,
    ) -> Result<(), ()> {
        let mut incoming_order = order;
        while let (false, Some(top_order)) = (
            incoming_order.is_closed(),
            self.peek_mut(&incoming_order.side().opposite()),
        ) {
            debug_assert!(
                !top_order.is_closed(),
                "top order cannot be closed before try to match"
            );

            if let Some(_trade) = incoming_order.trade(top_order) {
                if top_order.is_closed() {
                    // As long as top order is completed, it can be safely
                    // removed from orderbook.
                    self.pop(&incoming_order.side().opposite()).expect(
                        "Remove top order because it is completed already.",
                    );
                }

                if incoming_order.is_closed() {
                    // As long as incoming order is completed, it can be
                    // safely removed from
                    // orderbook.
                    break;
                }
            } else {
                // Since incoming order is not matching to top order
                // anymore, we can move on.
                break;
            }
        }

        // We need to check if incoming order is fullfilled. If not, we'll
        // insert it into orderbook.
        if !incoming_order.is_closed()
            && !incoming_order.is_immediate_or_cancel()
        {
            self.insert(incoming_order);
        }

        Ok(())
    }
}
