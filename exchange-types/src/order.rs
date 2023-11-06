use std::borrow::Borrow;
use std::cmp::{Ordering, Reverse};
use std::ops::AddAssign;

use exchange_core::Asset;
use thiserror::Error;

use crate::kind::TimeInForce;
use crate::trade::{PriceError, SideError, StatusError, TradeError};
use crate::{Id, Kind, Side, Status, Trade};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Order {
    id: Id,
    side: Side,
    #[cfg_attr(feature = "serde", serde(flatten))]
    kind: Kind,
    status: Status,
}

impl Order {
    #[inline]
    pub fn new(id: Id, side: Side, kind: Kind) -> Self {
        Self {
            id,
            side,
            kind,
            status: Status::Open,
        }
    }

    #[inline]
    pub fn new_limit(
        id: Id,
        side: Side,
        limit_price: u64,
        amount: u64,
    ) -> Self {
        Self {
            id,
            side,
            kind: Kind::Limit {
                limit_price,
                time_in_force: Default::default(),
                amount,
                filled: 0,
            },
            status: Status::Open,
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
        let filled = match self.kind {
            Kind::Limit { ref mut filled, .. }
            | Kind::Market { ref mut filled, .. } => filled,
        };

        filled.add_assign(amount);

        self.status = if self.remaining() == 0 {
            Status::Completed
        } else {
            Status::Partial
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
    type OrderId = Id;
    type OrderPrice = u64;
    type OrderSide = Side;
    type OrderStatus = Status;
    type Trade = Trade;
    type TradeError = TradeError;

    #[inline]
    fn id(&self) -> Id {
        self.id
    }

    #[inline]
    fn side(&self) -> Side {
        self.side
    }

    #[inline]
    fn limit_price(&self) -> Option<Self::OrderPrice> {
        match self.kind {
            Kind::Limit { limit_price, .. } => Some(limit_price),
            _ => None,
        }
    }

    #[inline]
    fn remaining(&self) -> Self::OrderAmount {
        match self.kind {
            Kind::Limit { amount, filled, .. }
            | Kind::Market { amount, filled, .. } => amount - filled,
        }
    }

    #[inline]
    fn status(&self) -> Status {
        self.status
    }

    #[inline]
    fn is_fill_or_kill(&self) -> bool {
        match self.kind {
            Kind::Market { all_or_none, .. }
            | Kind::Limit {
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
            Status::Cancelled | Status::Closed | Status::Completed
        )
    }

    #[inline]
    fn is_immediate_or_cancel(&self) -> bool {
        matches!(
            self.kind,
            Kind::Limit {
                time_in_force: TimeInForce::ImmediateOrCancel { .. },
                ..
            } | Kind::Market { .. }
        )
    }

    #[inline]
    fn is_post_only(&self) -> bool {
        matches!(self.kind, Kind::Limit { time_in_force: TimeInForce::GoodTilCancel { post_only }, .. } if post_only)
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
            (Side::Ask, Side::Bid)
                if taker.limit_price().is_some()
                    && taker.limit_price() > maker.limit_price() =>
            {
                Err(PriceError::Incompatible)?;
            }
            (Side::Bid, Side::Ask)
                if taker.limit_price().is_some()
                    && taker.limit_price() < maker.limit_price() =>
            {
                Err(PriceError::Incompatible)?;
            }
            (Side::Ask, Side::Bid) | (Side::Bid, Side::Ask) => (),
            _ => {
                Err(SideError::Conflict)?;
            }
        }

        Ok(())
    }

    #[inline]
    fn cancel(&mut self) {
        match self.status() {
            Status::Open => self.status = Status::Cancelled,
            Status::Partial => self.status = Status::Closed,
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

#[cfg(test)]
mod tests {
    use super::*;

    mod valid_trades {
        use super::*;

        #[test]
        fn same_prices() {
            let mut ask = Order::new_limit(Id::new(1), Side::Ask, 10, 10);
            let mut bid = Order::new_limit(Id::new(2), Side::Bid, 10, 10);

            assert!(ask.trade(&mut bid).is_ok());
        }

        #[test]
        fn different_prices() {
            let mut ask = Order::new_limit(Id::new(1), Side::Ask, 10, 10);
            let mut bid = Order::new_limit(Id::new(2), Side::Bid, 20, 10);

            assert!(ask.trade(&mut bid).is_ok());
        }

        #[test]
        fn partial_maker() {
            let mut ask = Order::new_limit(Id::new(1), Side::Ask, 10, 5);
            let mut bid = Order::new_limit(Id::new(2), Side::Bid, 20, 10);

            assert!(ask.trade(&mut bid).is_ok());
            assert!(ask.is_closed());
            assert!(!bid.is_closed());
        }

        #[test]
        fn partial_taker() {
            let mut ask = Order::new_limit(Id::new(1), Side::Ask, 10, 10);
            let mut bid = Order::new_limit(Id::new(2), Side::Bid, 20, 5);

            assert!(ask.trade(&mut bid).is_ok());
            assert!(!ask.is_closed());
            assert!(bid.is_closed());
        }
    }

    mod invalid_trades {
        use super::*;

        #[test]
        fn same_side() {
            let mut ask_1 = Order::new_limit(Id::new(1), Side::Ask, 10, 10);
            let mut ask_2 = Order::new_limit(Id::new(2), Side::Ask, 10, 10);

            assert!(ask_1.trade(&mut ask_2).is_err());
        }

        #[test]
        fn incompatible_prices() {
            let mut ask = Order::new_limit(Id::new(1), Side::Ask, 20, 10);
            let mut bid = Order::new_limit(Id::new(2), Side::Bid, 10, 10);

            assert!(ask.trade(&mut bid).is_err());
        }
    }

    #[test]
    fn cancel_order() {
        let mut ask = Order::new_limit(Id::new(1), Side::Ask, 10, 10);
        ask.cancel();
        assert_eq!(ask.status(), Status::Cancelled);
    }

    #[test]
    fn close_order() {
        let mut ask = Order::new_limit(Id::new(1), Side::Ask, 10, 10);
        let mut bid = Order::new_limit(Id::new(2), Side::Bid, 10, 5);

        assert!(ask.trade(&mut bid).is_ok());

        ask.cancel();

        assert_eq!(ask.status(), Status::Closed);
    }
}
