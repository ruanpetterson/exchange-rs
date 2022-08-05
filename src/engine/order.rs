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

impl Asset<BidOrder> for AskOrder {
    type OrderId = OrderId;
    type OrderStatus = OrderStatus;
    type OrderSide = OrderSide;
    type Trade = Trade;

    #[inline]
    fn id(&self) -> Self::OrderId {
        self.deref().id()
    }

    #[inline]
    fn side(&self) -> Self::OrderSide {
        self.deref().side()
    }

    #[inline]
    fn limit_price(&self) -> u64 {
        self.deref().limit_price()
    }

    #[inline]
    fn remaining(&self) -> u64 {
        self.deref().remaining()
    }

    #[inline]
    fn status(&self) -> Self::OrderStatus {
        self.deref().status()
    }

    #[inline]
    fn is_closed(&self) -> bool {
        self.deref().is_closed()
    }

    #[inline]
    fn trade(&mut self, order: &mut BidOrder) -> Option<Self::Trade> {
        self.deref_mut().trade(order)
    }

    #[inline]
    fn cancel(&mut self) {
        self.deref_mut().cancel()
    }
}

impl Asset<AskOrder> for BidOrder {
    type OrderId = OrderId;
    type OrderStatus = OrderStatus;
    type OrderSide = OrderSide;
    type Trade = Trade;

    #[inline]
    fn id(&self) -> Self::OrderId {
        self.deref().id()
    }

    #[inline]
    fn side(&self) -> Self::OrderSide {
        self.deref().side()
    }

    #[inline]
    fn limit_price(&self) -> u64 {
        self.deref().limit_price()
    }

    #[inline]
    fn remaining(&self) -> u64 {
        self.deref().remaining()
    }

    #[inline]
    fn status(&self) -> Self::OrderStatus {
        self.deref().status()
    }

    #[inline]
    fn is_closed(&self) -> bool {
        self.deref().is_closed()
    }

    #[inline]
    fn trade(&mut self, order: &mut AskOrder) -> Option<Self::Trade> {
        self.deref_mut().trade(order)
    }

    #[inline]
    fn cancel(&mut self) {
        self.deref_mut().cancel()
    }
}
