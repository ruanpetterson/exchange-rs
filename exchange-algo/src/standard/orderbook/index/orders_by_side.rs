use std::borrow::Borrow;
use std::collections::VecDeque;
use std::ops::{Deref, Index, IndexMut};

use either::Either;
use exchange_core::Asset;
use exchange_types::OrderSide;

use super::OrdersByPrice;

pub struct OrdersBySide<Order: Asset> {
    ask: OrdersByPrice<Order>,
    bid: OrdersByPrice<Order>,
}

impl<Order: Asset> OrdersBySide<Order>
where
    Order: Asset<OrderSide = OrderSide>,
{
    #[inline]
    pub fn iter(
        &self,
        side: &<Order as Asset>::OrderSide,
    ) -> impl Iterator<Item = &<Order as Asset>::OrderId> {
        match side {
            OrderSide::Ask => Either::Left(
                self[side].deref().values().flat_map(VecDeque::iter),
            ),
            OrderSide::Bid => Either::Right(
                self[side].deref().values().rev().flat_map(VecDeque::iter),
            ),
        }
    }

    #[inline]
    pub fn peek(
        &self,
        side: &<Order as Asset>::OrderSide,
    ) -> Option<&<Order as Asset>::OrderId> {
        self.iter(side).next()
    }
}
impl<Order: Asset> Default for OrdersBySide<Order> {
    #[inline]
    fn default() -> Self {
        Self {
            ask: Default::default(),
            bid: Default::default(),
        }
    }
}

impl<Order, S> Index<S> for OrdersBySide<Order>
where
    Order: Asset<OrderSide = OrderSide>,
    S: Borrow<<Order as Asset>::OrderSide>,
{
    type Output = OrdersByPrice<Order>;

    #[inline]
    fn index(&self, side: S) -> &Self::Output {
        match *side.borrow() {
            OrderSide::Ask => &self.ask,
            OrderSide::Bid => &self.bid,
        }
    }
}

impl<Order, S> IndexMut<S> for OrdersBySide<Order>
where
    Order: Asset<OrderSide = OrderSide>,
    S: Borrow<<Order as Asset>::OrderSide>,
{
    #[inline]
    fn index_mut(&mut self, side: S) -> &mut Self::Output {
        match side.borrow() {
            OrderSide::Ask => &mut self.ask,
            OrderSide::Bid => &mut self.bid,
        }
    }
}
