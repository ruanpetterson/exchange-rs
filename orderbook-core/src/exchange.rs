use crate::{Asset, Opposite};

/// An interface for dealing with exchange.
///
/// This is the core trait for exchange implementation.
pub trait Exchange {
    /// The type of order that will be stored in the exchange.
    type Order: Asset;

    /// Inserts an order into the exchange.
    fn insert(&mut self, order: Self::Order);

    /// Removes an order from the exchange.
    fn remove(
        &mut self,
        order: &<Self::Order as Asset>::OrderId,
    ) -> Option<Self::Order>;

    /// Core exchange algorithm.
    fn matching(&mut self, order: Self::Order) {
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
                    // As long as incoming order is completed, it can be safely
                    // removed from orderbook.
                    break;
                }
            } else {
                // Since incoming order is not matching to top order anymore, we
                // can move on.
                break;
            }
        }

        // We need to check if incoming order is fullfilled. If not, we'll
        // insert it into orderbook.
        if !incoming_order.is_closed() {
            self.insert(incoming_order);
        }
    }

    /// Returns a reference of the most relevant order in the exchange.
    fn peek(
        &self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> Option<&Self::Order>;

    /// Returns a mutable reference of the most relevant order in the exchange.
    fn peek_mut(
        &mut self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> Option<&mut Self::Order>;

    /// Removes the most relevant order in the exchange.
    fn pop(
        &mut self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> Option<Self::Order>;
}

pub trait ExchangeExt: Exchange {
    /// Returns the difference or gap that exists between bid and ask
    /// prices.
    fn spread(&self) -> Option<(u64, u64)>;

    /// Returns the number of shares being bid on or offered.
    fn len(&self) -> (usize, usize);

    /// Returns `true` if the exchange contains no items.
    fn is_empty(&self) -> bool {
        self.len() == (0, 0)
    }
}
