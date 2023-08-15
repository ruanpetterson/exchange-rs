use std::borrow::Borrow;
use std::cmp::{Ordering, Reverse};
use std::ops::{Deref, DerefMut};

use orderbook_core::{Asset, OrderSide};
use thiserror::Error;

use crate::order_type::TimeInForce;
use crate::{OrderId, OrderStatus, OrderType, Trade};

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("fill exceeds remaning amount")]
    Overfill,
    #[error("sides mismatch")]
    MismatchSide,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Order {
    id: OrderId,
    side: OrderSide,
    #[cfg_attr(feature = "serde", serde(flatten))]
    type_: OrderType,
    amount: u64,
    #[cfg_attr(feature = "serde", serde(default))]
    filled: u64,
    status: OrderStatus,
}

impl Order {
    #[inline]
    pub fn new(
        id: OrderId,
        side: OrderSide,
        type_: OrderType,
        amount: u64,
    ) -> Self {
        Self {
            id,
            side,
            type_,
            amount,
            filled: 0,
            status: OrderStatus::Open,
        }
    }

    #[inline]
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
            },
            amount,
            filled: 0,
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
    /// This results in undefined behavior when current `Order::filled`
    /// overflows `Order::amount`.
    #[inline]
    pub(crate) unsafe fn fill_unchecked(&mut self, amount: u64) {
        self.filled += amount;
        self.status = if self.filled == self.amount {
            OrderStatus::Completed
        } else {
            OrderStatus::Partial
        };
    }

    /// Fill an order within the specified amount, returning an error if
    /// something fails.
    #[inline]
    pub(crate) fn try_fill(&mut self, amount: u64) -> Result<(), OrderError> {
        if amount > self.remaining() {
            return Err(OrderError::Overfill);
        }

        // SAFETY: we already checked that `remaining >= amount`.
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
        let ord = if self.id.eq(&other.id) {
            Ordering::Equal
        } else {
            self.limit_price()?.cmp(&other.limit_price()?)
        };

        Some(ord)
    }
}

impl Asset for Order {
    type OrderAmount = u64;
    type OrderId = OrderId;
    type OrderPrice = u64;
    type OrderSide = OrderSide;
    type OrderStatus = OrderStatus;
    type Trade = Trade;

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
        self.amount - self.filled
    }

    #[inline]
    fn status(&self) -> OrderStatus {
        self.status
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
    fn trade(&mut self, other: &mut Self) -> Option<Self::Trade> {
        let (taker, maker) = (self, other);

        // TODO: provide more useuful outputs to matching algorithms.
        Trade::new(taker, maker).ok()
    }

    #[inline]
    fn matches(&self, order: &Self) -> bool {
        let (taker, maker) = (self, order);

        // Matching cannot occur between closed orders.
        if taker.is_closed() || maker.is_closed() {
            return false;
        }

        match (taker.side(), maker.side()) {
            (OrderSide::Ask, OrderSide::Bid) => {
                matches!(taker.type_, OrderType::Market) || taker <= maker
            }
            (OrderSide::Bid, OrderSide::Ask) => {
                matches!(taker.type_, OrderType::Market) || taker >= maker
            }
            _ => false,
        }
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

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct AskOrder(Order);

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct BidOrder(Order);

impl TryFrom<Order> for AskOrder {
    type Error = OrderError;

    #[inline]
    fn try_from(order: Order) -> Result<Self, Self::Error> {
        order
            .side()
            .eq(&OrderSide::Ask)
            .then_some(Self(order))
            .ok_or(OrderError::MismatchSide)
    }
}

impl TryFrom<Order> for BidOrder {
    type Error = OrderError;

    #[inline]
    fn try_from(order: Order) -> Result<Self, Self::Error> {
        order
            .side()
            .eq(&OrderSide::Bid)
            .then_some(Self(order))
            .ok_or(OrderError::MismatchSide)
    }
}

impl From<AskOrder> for Order {
    #[inline]
    fn from(order: AskOrder) -> Self {
        order.0
    }
}

impl From<BidOrder> for Order {
    #[inline]
    fn from(order: BidOrder) -> Self {
        order.0
    }
}

impl Deref for AskOrder {
    type Target = Order;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AskOrder {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for BidOrder {
    type Target = Order;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BidOrder {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_into_ask() {
        let ask_orders = [OrderSide::Ask, OrderSide::Bid]
            .into_iter()
            .enumerate()
            .filter_map(|(id, side)| {
                Order::new_limit(OrderId::new(id as u64), side, 100, 100)
                    .try_into()
                    .ok()
            })
            .collect::<Vec<AskOrder>>();
        assert_eq!(ask_orders.len(), 1);

        let orders = ask_orders
            .into_iter()
            .map(|order| order.into())
            .collect::<Vec<Order>>();
        assert_eq!(orders.len(), 1);
    }

    #[test]
    fn try_into_bid() {
        let bid_orders = [OrderSide::Ask, OrderSide::Bid]
            .into_iter()
            .enumerate()
            .filter_map(|(id, side)| {
                Order::new_limit(OrderId::new(id as u64), side, 100, 100)
                    .try_into()
                    .ok()
            })
            .collect::<Vec<BidOrder>>();
        assert_eq!(bid_orders.len(), 1);

        let orders = bid_orders
            .into_iter()
            .map(|order| order.into())
            .collect::<Vec<Order>>();
        assert_eq!(orders.len(), 1);
    }

    mod valid_trades {
        use super::*;

        #[test]
        fn same_prices() {
            let mut ask =
                Order::new_limit(OrderId::new(1), OrderSide::Ask, 10, 10);
            let mut bid =
                Order::new_limit(OrderId::new(2), OrderSide::Bid, 10, 10);

            assert!(ask.trade(&mut bid).is_some());
        }

        #[test]
        fn different_prices() {
            let mut ask =
                Order::new_limit(OrderId::new(1), OrderSide::Ask, 10, 10);
            let mut bid =
                Order::new_limit(OrderId::new(2), OrderSide::Bid, 20, 10);

            assert!(ask.trade(&mut bid).is_some());
        }

        #[test]
        fn partial_maker() {
            let mut ask =
                Order::new_limit(OrderId::new(1), OrderSide::Ask, 10, 5);
            let mut bid =
                Order::new_limit(OrderId::new(2), OrderSide::Bid, 20, 10);

            assert!(ask.trade(&mut bid).is_some());
            assert!(ask.is_closed());
            assert!(!bid.is_closed());
        }

        #[test]
        fn partial_taker() {
            let mut ask =
                Order::new_limit(OrderId::new(1), OrderSide::Ask, 10, 10);
            let mut bid =
                Order::new_limit(OrderId::new(2), OrderSide::Bid, 20, 5);

            assert!(ask.trade(&mut bid).is_some());
            assert!(!ask.is_closed());
            assert!(bid.is_closed());
        }
    }

    mod invalid_trades {
        use super::*;

        #[test]
        fn same_side() {
            let mut ask_1 =
                Order::new_limit(OrderId::new(1), OrderSide::Ask, 10, 10);
            let mut ask_2 =
                Order::new_limit(OrderId::new(2), OrderSide::Ask, 10, 10);

            assert!(ask_1.trade(&mut ask_2).is_none());
        }

        #[test]
        fn incompatible_prices() {
            let mut ask =
                Order::new_limit(OrderId::new(1), OrderSide::Ask, 20, 10);
            let mut bid =
                Order::new_limit(OrderId::new(2), OrderSide::Bid, 10, 10);

            assert!(ask.trade(&mut bid).is_none());
        }
    }

    #[test]
    fn cancel_order() {
        let mut ask = Order::new_limit(OrderId::new(1), OrderSide::Ask, 10, 10);
        ask.cancel();
        assert_eq!(ask.status(), OrderStatus::Cancelled);
    }

    #[test]
    fn close_order() {
        let mut ask = Order::new_limit(OrderId::new(1), OrderSide::Ask, 10, 10);
        let mut bid = Order::new_limit(OrderId::new(2), OrderSide::Bid, 10, 5);

        assert!(ask.trade(&mut bid).is_some());

        ask.cancel();

        assert_eq!(ask.status(), OrderStatus::Closed);
    }
}
