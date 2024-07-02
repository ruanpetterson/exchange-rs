use std::ops::Deref;
use std::ops::DerefMut;

use crate::Algo;
use crate::Asset;
use crate::Trade;

pub type Spread<Order> =
    (<Order as Asset>::OrderPrice, <Order as Asset>::OrderPrice);
pub type Volume<Order> = (
    <Order as Asset>::OrderQuantity,
    <Order as Asset>::OrderQuantity,
);

/// An interface for dealing with exchange.
///
/// This is the core trait for exchange implementation.
pub trait Exchange {
    type Algo<O>: Algo<O>;
    /// The type of order that will be stored in the exchange.
    type Order: Asset;
    type OrderRef<'e>: Deref<Target = Self::Order>
    where
        Self: 'e;
    type OrderRefMut<'e>: DerefMut<Target = Self::Order>
    where
        Self: 'e;

    // Returns an iterator over the given side of the exchange.
    fn iter(
        &self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> impl Iterator<Item = Self::OrderRef<'_>> + '_;

    /// Inserts an order into the exchange.
    ///
    /// # Safety
    ///
    /// This is a unsafe call because the caller need to take extra care to
    /// ensure the integrity and safety of our orderbook by guaranteeing
    /// that the insertion operation will not result in any overlap between
    /// the sides of the orderbook.
    ///
    /// This method is inteded to be used at `Exchange::matching` internals.
    unsafe fn insert(&mut self, order: Self::Order);

    /// Removes an order from the exchange.
    fn remove(
        &mut self,
        side: &<Self::Order as Asset>::OrderSide,
        level: &<Self::Order as Asset>::OrderPrice,
        order_id: &<Self::Order as Asset>::OrderId,
    ) -> Option<Self::Order>;

    /// Returns a reference of the most relevant order in the exchange.
    fn peek(
        &self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> Option<Self::OrderRef<'_>>;

    /// Returns a mutable reference of the most relevant order in the exchange.
    fn peek_mut(
        &mut self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> Option<Self::OrderRefMut<'_>>;

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
    // TODO: Consider creating a dedicated `Error` type for `Exchange`, and make
    //       it mandatory for concrete types to implement this method.
    #[allow(clippy::type_complexity)]
    fn matching<O>(
        &mut self,
        incoming_order: O,
    ) -> Result<
        <Self::Algo<O> as Algo<O>>::Output,
        <Self::Algo<O> as Algo<O>>::Error,
    >
    where
        Self: ExchangeExt + Sized,
        Self::Order: Trade<O> + TryFrom<O>,
        O: Asset<
            OrderId = <<Self as Exchange>::Order as Asset>::OrderId,
            OrderNotional = <<Self as Exchange>::Order as Asset>::OrderNotional,
            OrderPrice = <<Self as Exchange>::Order as Asset>::OrderPrice,
            OrderQuantity = <<Self as Exchange>::Order as Asset>::OrderQuantity,
            OrderSide = <<Self as Exchange>::Order as Asset>::OrderSide,
            OrderStatus = <<Self as Exchange>::Order as Asset>::OrderStatus,
        >,
    {
        <Self::Algo<O> as Algo<O>>::matching(self, incoming_order)
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
