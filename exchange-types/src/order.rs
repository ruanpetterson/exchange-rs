use std::borrow::Borrow;
use std::cmp::{Ordering, Reverse};
use std::ops::AddAssign;

use exchange_core::Asset;
use thiserror::Error;

use crate::order_type::TimeInForce;
use crate::trade::{PriceError, SideError, StatusError, TradeError};
use crate::{OrderId, OrderSide, OrderStatus, OrderType, Trade};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Order {
    id: OrderId,
    side: OrderSide,
    #[cfg_attr(feature = "serde", serde(flatten))]
    type_: OrderType,
    status: OrderStatus,
}

impl Order {
    #[inline]
    pub fn new(id: OrderId, side: OrderSide, type_: OrderType) -> Self {
        Self {
            id,
            side,
            type_,
            status: OrderStatus::Open,
        }
    }

    #[inline]
    #[cfg(any(test, feature = "test"))]
    pub fn builder() -> builder::Builder<(), ()> {
        builder::Builder::new()
    }

    #[inline]
    #[deprecated]
    pub fn new_limit(
        id: OrderId,
        side: OrderSide,
        limit_price: u64,
        amount: u64,
    ) -> Self {
        Self {
            id,
            side,
            type_: OrderType::Limit {
                limit_price,
                time_in_force: Default::default(),
                amount,
                filled: 0,
            },
            status: OrderStatus::Open,
        }
    }

    /// Fill an order within the specified amount.
    ///
    /// # Panics
    ///
    /// Panics if `amount` is greater then `remaining`.
    #[inline]
    pub(crate) fn fill(&mut self, amount: u64) {
        self.try_fill(amount)
            .expect("order does not have available amount to fill")
    }

    /// Fill an order within the specified amount.
    ///
    /// # Safety
    ///
    /// This results in an unreliable state when current `Order::filled`
    /// overflows `Order::amount` or given amount is zero.
    #[inline]
    pub(crate) unsafe fn fill_unchecked(&mut self, amount: u64) {
        let filled = match self.type_ {
            OrderType::Limit { ref mut filled, .. }
            | OrderType::Market { ref mut filled, .. } => filled,
        };

        filled.add_assign(amount);

        self.status = if self.remaining() == 0 {
            OrderStatus::Completed
        } else {
            OrderStatus::Partial
        };
    }

    /// Fill an order within the specified amount, returning an error if
    /// something fails.
    #[inline]
    pub(crate) fn try_fill(&mut self, amount: u64) -> Result<(), OrderError> {
        if amount == 0 {
            return Err(OrderError::NoFill);
        }

        if amount > self.remaining() {
            return Err(OrderError::Overfill);
        }

        // SAFETY: we already guarantee that `remaining >= amount > 0`.
        unsafe { self.fill_unchecked(amount) };

        Ok(())
    }
}

impl Borrow<Order> for Reverse<Order> {
    #[inline]
    fn borrow(&self) -> &Order {
        &self.0
    }
}

impl PartialEq for Order {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}
impl Eq for Order {}

impl PartialOrd for Order {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.limit_price().partial_cmp(&other.limit_price())
    }
}

impl Asset for Order {
    type OrderAmount = u64;
    type OrderId = OrderId;
    type OrderPrice = u64;
    type OrderSide = OrderSide;
    type OrderStatus = OrderStatus;
    type Trade = Trade;
    type TradeError = TradeError;

    #[inline]
    fn id(&self) -> OrderId {
        self.id
    }

    #[inline]
    fn side(&self) -> OrderSide {
        self.side
    }

    #[inline]
    fn limit_price(&self) -> Option<Self::OrderPrice> {
        match self.type_ {
            OrderType::Limit { limit_price, .. } => Some(limit_price),
            _ => None,
        }
    }

    #[inline]
    fn remaining(&self) -> Self::OrderAmount {
        match self.type_ {
            OrderType::Limit { amount, filled, .. }
            | OrderType::Market { amount, filled, .. } => amount - filled,
        }
    }

    #[inline]
    fn status(&self) -> OrderStatus {
        self.status
    }

    #[inline]
    fn is_fill_or_kill(&self) -> bool {
        match self.type_ {
            OrderType::Market { all_or_none, .. }
            | OrderType::Limit {
                time_in_force: TimeInForce::ImmediateOrCancel { all_or_none },
                ..
            } => all_or_none,
            _ => false,
        }
    }

    #[inline]
    fn is_closed(&self) -> bool {
        matches!(
            self.status(),
            OrderStatus::Cancelled
                | OrderStatus::Closed
                | OrderStatus::Completed
        )
    }

    #[inline]
    fn is_immediate_or_cancel(&self) -> bool {
        matches!(
            self.type_,
            OrderType::Limit {
                time_in_force: TimeInForce::ImmediateOrCancel { .. },
                ..
            } | OrderType::Market { .. }
        )
    }

    #[inline]
    fn is_post_only(&self) -> bool {
        matches!(self.type_, OrderType::Limit { time_in_force: TimeInForce::GoodTilCancel { post_only }, .. } if post_only)
    }

    #[inline]
    fn trade(
        &mut self,
        other: &mut Self,
    ) -> Result<Self::Trade, Self::TradeError> {
        let (taker, maker) = (self, other);

        Trade::new(taker, maker)
    }

    #[inline]
    fn matches(&self, other: &Self) -> Result<(), Self::TradeError> {
        let (taker, maker) = (self, other);

        // Matching cannot occur between closed orders.
        if taker.is_closed() || maker.is_closed() {
            Err(StatusError::Closed)?
        }

        match (taker.side(), maker.side()) {
            (OrderSide::Ask, OrderSide::Bid)
                if taker.limit_price().is_some()
                    && taker.limit_price() > maker.limit_price() =>
            {
                Err(PriceError::Incompatible)?;
            }
            (OrderSide::Bid, OrderSide::Ask)
                if taker.limit_price().is_some()
                    && taker.limit_price() < maker.limit_price() =>
            {
                Err(PriceError::Incompatible)?;
            }
            (OrderSide::Ask, OrderSide::Bid)
            | (OrderSide::Bid, OrderSide::Ask) => (),
            _ => {
                Err(SideError::Conflict)?;
            }
        }

        Ok(())
    }

    #[inline]
    fn cancel(&mut self) {
        match self.status() {
            OrderStatus::Open => self.status = OrderStatus::Cancelled,
            OrderStatus::Partial => self.status = OrderStatus::Closed,
            _ => (),
        }
    }
}

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("filling amount exceeds remaining amount")]
    Overfill,
    #[error("empty filling is not allowed")]
    NoFill,
}

#[cfg(any(test, feature = "test"))]
mod builder {
    use super::*;

    pub struct Builder<T, S> {
        side: S,
        type_: T,
    }

    impl Builder<(), ()> {
        #[inline]
        pub const fn new() -> Self {
            Self {
                side: (),
                type_: (),
            }
        }
    }

    impl<T, S> Builder<T, S> {
        #[inline]
        pub fn side(self, side: OrderSide) -> Builder<T, OrderSide> {
            Builder {
                side,
                type_: self.type_,
            }
        }

        #[inline]
        pub fn limit(
            self,
            limit_price: u64,
            amount: u64,
        ) -> Builder<OrderType, S> {
            Builder {
                side: self.side,
                type_: OrderType::Limit {
                    limit_price,
                    time_in_force: TimeInForce::default(),
                    amount,
                    filled: 0,
                },
            }
        }
    }

    impl Builder<OrderType, OrderSide> {
        #[inline]
        pub fn build(self) -> Order {
            Order {
                id: OrderId::random(),
                side: self.side,
                type_: self.type_,
                status: OrderStatus::Open,
            }
        }
    }

    impl From<Builder<OrderType, OrderSide>> for Order {
        #[inline]
        fn from(builder: Builder<OrderType, OrderSide>) -> Self {
            builder.build()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod valid_trades {
        use super::*;

        #[test]
        fn same_prices() {
            let mut ask =
                Order::builder().side(OrderSide::Ask).limit(10, 10).build();
            let mut bid =
                Order::builder().side(OrderSide::Bid).limit(10, 10).build();

            assert!(ask.trade(&mut bid).is_ok());
        }

        #[test]
        fn different_prices() {
            let mut ask =
                Order::builder().side(OrderSide::Ask).limit(10, 10).build();
            let mut bid =
                Order::builder().side(OrderSide::Bid).limit(20, 10).build();

            assert!(ask.trade(&mut bid).is_ok());
        }

        #[test]
        fn partial_maker() {
            let mut ask =
                Order::builder().side(OrderSide::Ask).limit(10, 5).build();
            let mut bid =
                Order::builder().side(OrderSide::Bid).limit(20, 10).build();

            assert!(ask.trade(&mut bid).is_ok());
            assert!(ask.is_closed());
            assert!(!bid.is_closed());
        }

        #[test]
        fn partial_taker() {
            let mut ask =
                Order::builder().side(OrderSide::Ask).limit(10, 10).build();
            let mut bid =
                Order::builder().side(OrderSide::Bid).limit(20, 5).build();

            assert!(ask.trade(&mut bid).is_ok());
            assert!(!ask.is_closed());
            assert!(bid.is_closed());
        }
    }

    mod invalid_trades {
        use super::*;

        #[test]
        fn same_side() {
            let mut ask_1 =
                Order::builder().side(OrderSide::Ask).limit(10, 10).build();
            let mut ask_2 =
                Order::builder().side(OrderSide::Ask).limit(10, 10).build();

            assert!(ask_1.trade(&mut ask_2).is_err());
        }

        #[test]
        fn incompatible_prices() {
            let mut ask =
                Order::builder().side(OrderSide::Ask).limit(20, 10).build();
            let mut bid =
                Order::builder().side(OrderSide::Bid).limit(10, 10).build();

            assert!(ask.trade(&mut bid).is_err());
        }
    }

    #[test]
    fn cancel_order() {
        let mut ask =
            Order::builder().side(OrderSide::Ask).limit(10, 10).build();
        ask.cancel();
        assert_eq!(ask.status(), OrderStatus::Cancelled);
    }

    #[test]
    fn close_order() {
        let mut ask =
            Order::builder().side(OrderSide::Ask).limit(10, 10).build();
        let mut bid =
            Order::builder().side(OrderSide::Bid).limit(10, 5).build();

        assert!(ask.trade(&mut bid).is_ok());

        ask.cancel();

        assert_eq!(ask.status(), OrderStatus::Closed);
    }
}
