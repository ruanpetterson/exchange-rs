use std::error::Error;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

use either::Either;
use num::Zero;

pub trait Asset: PartialOrd {
    /// Order unique identifier.
    type OrderId: Copy + Eq + Ord;
    type OrderNotional: Div<Self::OrderQuantity, Output = Self::OrderPrice>
        + Div<Self::OrderPrice, Output = Self::OrderQuantity>
        + Sub<Output = Self::OrderNotional>
        + Copy
        + Ord
        + Zero;
    /// Order price.
    type OrderPrice: Mul<Self::OrderQuantity, Output = Self::OrderNotional>
        + Copy
        + Ord;
    /// Order quantity.
    type OrderQuantity: Add<Output = Self::OrderQuantity>
        + Sub<Output = Self::OrderQuantity>
        + Mul<Self::OrderPrice, Output = Self::OrderNotional>
        + Copy
        + Ord
        + Zero;
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
    fn remaining(&self) -> Either<Self::OrderNotional, Self::OrderQuantity>;
    /// Return current order status.
    fn status(&self) -> Self::OrderStatus;
    /// Returns `true` if order is fill or kill.
    fn is_fill_or_kill(&self) -> bool;
    /// Returns `true` if order is open.
    fn is_open(&self) -> bool;
    /// Returns `true` if order is closed.
    fn is_closed(&self) -> bool;
    /// Returns `true` if order is immediate or cancel.
    fn is_immediate_or_cancel(&self) -> bool;
    /// Returns `true` if order is post-only.
    fn is_post_only(&self) -> bool;
    /// Cancel the order.
    fn cancel(&mut self);
}

pub trait Trade<Rhs>: Asset
where
    Rhs: Asset,
{
    /// Execute a trade.
    fn trade(
        &mut self,
        other: &mut Rhs,
    ) -> Result<Self::Trade, Self::TradeError>;
    /// Returns `Ok` if orders match.
    fn matches(&self, other: &Rhs) -> Result<(), Self::TradeError>;
}

/// The logical opposite of a value.
pub trait Opposite<Opposite = Self> {
    /// Returns the opposite value.
    fn opposite(&self) -> Opposite;
}
