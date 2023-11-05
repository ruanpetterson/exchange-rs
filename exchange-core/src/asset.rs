use std::error::Error;
use std::ops::Add;

use num::Zero;

pub trait Asset<Order = Self>: PartialOrd {
    /// Order amount.
    type OrderAmount: Add<Output = Self::OrderAmount> + Copy + Ord + Zero;
    /// Order unique identifier.
    type OrderId: Copy + Eq;
    /// Order price.
    type OrderPrice: Copy + Ord;
    /// Order side.
    type OrderSide: Opposite;
    /// Order current status.
    type OrderStatus: Copy + Eq;
    /// Trade struct.
    type Trade;
    /// Trade error struct.
    type TradeError: Error;
    /// Return order unique identifier.
    fn id(&self) -> Self::OrderId;
    /// Return order side.
    fn side(&self) -> Self::OrderSide;
    /// Return order limit price.
    fn limit_price(&self) -> Option<Self::OrderPrice>;
    /// Return order remaining amount.
    fn remaining(&self) -> Self::OrderAmount;
    /// Return current order status.
    fn status(&self) -> Self::OrderStatus;
    /// Returns `true` if order is fill or kill.
    fn is_fill_or_kill(&self) -> bool;
    /// Returns `true` if order is closed.
    fn is_closed(&self) -> bool;
    /// Returns `true` if order is immediate or cancel.
    fn is_immediate_or_cancel(&self) -> bool;
    /// Returns `true` if order is post-only.
    fn is_post_only(&self) -> bool;
    /// Returns `Ok` if orders match.
    fn matches(&self, other: &Order) -> Result<(), Self::TradeError>;
    /// Execute a trade.
    fn trade(
        &mut self,
        other: &mut Order,
    ) -> Result<Self::Trade, Self::TradeError>;
    /// Cancel the order.
    fn cancel(&mut self);
}

/// The logical opposite of a value.
pub trait Opposite<Opposite = Self> {
    /// Returns the opposite value.
    fn opposite(&self) -> Opposite;
}
