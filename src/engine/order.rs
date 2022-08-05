use std::borrow::Borrow;
use std::cmp::{Ordering, Reverse};
use std::ops::{Deref, DerefMut};

use super::{OrderId, OrderStatus, Trade};
use crate::{Asset, OrderSide};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("sides mismatch")]
    MismatchSide,
}

#[derive(Debug)]
#[cfg_attr(test, derive(Copy, Clone))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Order {
    id: OrderId,
    account_id: u64,
    side: OrderSide,
    limit_price: u64,
    amount: u64,
    #[cfg_attr(feature = "serde", serde(default))]
    filled: u64,
    status: OrderStatus,
}

impl Order {
    #[inline]
    pub fn new(
        id: OrderId,
        account_id: u64,
        side: OrderSide,
        limit_price: u64,
        amount: u64,
    ) -> Self {
        Self {
            id,
            account_id,
            side,
            limit_price,
            amount,
            filled: 0,
            status: OrderStatus::Open,
        }
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
        Some(self.cmp(other))
    }
}

impl Ord for Order {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if self.id.eq(&other.id) {
            Ordering::Equal
        } else {
            self.limit_price.cmp(&other.limit_price)
        }
    }
}

impl Asset for Order {
    type OrderId = OrderId;
    type OrderSide = OrderSide;
    type OrderStatus = OrderStatus;
    type Trade = Trade;

    #[inline]
    fn id(&self) -> OrderId {
        self.id
    }

    #[inline]
    fn status(&self) -> OrderStatus {
        self.status
    }

    #[inline]
    fn side(&self) -> OrderSide {
        self.side
    }

    #[inline]
    fn limit_price(&self) -> u64 {
        self.limit_price
    }

    #[inline]
    fn remaining(&self) -> u64 {
        self.amount - self.filled
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
    fn trade(&mut self, other: &mut Self) -> Option<Self::Trade> {
        let (taker, maker) = (self, other);

        #[inline(always)]
        fn matches_with(taker: &Order, maker: &Order) -> bool {
            match (taker.side(), maker.side()) {
                (OrderSide::Ask, OrderSide::Bid) => taker <= maker,
                (OrderSide::Bid, OrderSide::Ask) => taker >= maker,
                _ => false,
            }
        }

        #[inline(always)]
        fn subtract_amount(order: &mut Order, exchanged: u64) {
            debug_assert!(
                order.remaining() >= exchanged,
                "exchanged amount should be less or equal to remaining"
            );

            order.filled += exchanged;
            order.status = if order.filled == order.amount {
                OrderStatus::Completed
            } else {
                OrderStatus::Partial
            };
        }

        matches_with(taker, maker).then(|| {
            let exchanged = taker.remaining().min(maker.remaining());
            let price = match taker.side() {
                OrderSide::Ask => taker.limit_price().max(maker.limit_price()),
                OrderSide::Bid => taker.limit_price().min(maker.limit_price()),
            };
            subtract_amount(taker, exchanged);
            subtract_amount(maker, exchanged);

            Trade {
                taker: taker.id,
                maker: maker.id,
                amount: exchanged,
                price,
            }
        })
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AskOrder(Order);

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BidOrder(Order);

impl TryFrom<Order> for AskOrder {
    type Error = OrderError;

    #[inline]
    fn try_from(order: Order) -> Result<Self, Self::Error> {
        order
            .side()
            .eq(&OrderSide::Ask)
            .then(|| Self(order))
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
            .then(|| Self(order))
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
                Order::new(OrderId::new(id as u64), 1, side, 100, 100)
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
                Order::new(OrderId::new(id as u64), 1, side, 100, 100)
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

    #[test]
    fn valid_trade() {
        let mut ask = Order::new(OrderId::new(1), 1, OrderSide::Ask, 10, 10);
        let mut bid = Order::new(OrderId::new(2), 1, OrderSide::Bid, 10, 10);

        assert!(ask.trade(&mut bid).is_some());
    }

    #[test]
    fn invalid_trade() {
        let mut ask_1 = Order::new(OrderId::new(1), 1, OrderSide::Ask, 10, 10);
        let mut ask_2 = Order::new(OrderId::new(2), 1, OrderSide::Ask, 10, 10);

        assert!(ask_1.trade(&mut ask_2).is_none());
    }

    #[test]
    fn cancel_order() {
        let mut ask = Order::new(OrderId::new(1), 1, OrderSide::Ask, 10, 10);
        ask.cancel();
        assert_eq!(ask.status(), OrderStatus::Cancelled);
    }

    #[test]
    fn close_order() {
        let mut ask = Order::new(OrderId::new(1), 1, OrderSide::Ask, 10, 10);
        let mut bid = Order::new(OrderId::new(2), 1, OrderSide::Bid, 10, 5);

        assert!(ask.trade(&mut bid).is_some());

        ask.cancel();

        assert_eq!(ask.status(), OrderStatus::Closed);
    }
}
