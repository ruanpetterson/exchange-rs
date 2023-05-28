use crate::Asset;

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
    fn spread(
        &self,
    ) -> Option<(
        <Self::Order as Asset>::OrderPrice,
        <Self::Order as Asset>::OrderPrice,
    )>;

    /// Returns the number of shares being bid on or offered.
    fn len(&self) -> (usize, usize);

    /// Returns `true` if the exchange contains no items.
    fn is_empty(&self) -> bool {
        self.len() == (0, 0)
    }
}
