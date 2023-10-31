use crate::{Algo, Asset};

pub type Spread<Order> =
    (<Order as Asset>::OrderPrice, <Order as Asset>::OrderPrice);
pub type Volume<Order> =
    (<Order as Asset>::OrderAmount, <Order as Asset>::OrderAmount);

/// An interface for dealing with exchange.
///
/// This is the core trait for exchange implementation.
pub trait Exchange {
    /// The type of order that will be stored in the exchange.
    type Algo: Algo;
    type Order: Asset;

    // Returns an iterator over the given side of the exchange.
    fn iter(
        &self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> impl Iterator<Item = &Self::Order>;

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

    /// Attempt to match an incoming order.
    ///
    /// This method takes an order as input and attempts to match it against the
    /// existing limit orders in the orderbook. Matching is done in a specific
    /// order based on the orderbook's rules, such as price-time priority.
    fn matching(
        &mut self,
        incoming_order: Self::Order,
    ) -> Result<<Self::Algo as Algo>::Output, <Self::Algo as Algo>::Error>
    where
        Self: ExchangeExt + Sized,
    {
        <Self::Algo as Algo>::matching(self, incoming_order)
    }
}

pub trait ExchangeExt: Exchange {
    /// Returns the difference or gap that exists between bid and ask
    /// prices.
    fn spread(&self) -> Option<Spread<Self::Order>>;

    /// Returns the number of shares being bid on or offered.
    fn len(&self) -> (usize, usize);

    /// Returns `true` if the exchange contains no items.
    fn is_empty(&self) -> bool {
        self.len() == (0, 0)
    }

    fn volume(&self) -> Volume<Self::Order>;
}
