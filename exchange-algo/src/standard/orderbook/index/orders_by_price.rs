use std::borrow::Borrow;
use std::collections::{BTreeMap, VecDeque};
use std::ops::{Index, IndexMut};

use exchange_core::Asset;
use exchange_types::OrderSide;

pub struct OrdersByPrice<Order: Asset> {
    ask: BTreeMap<
        <Order as Asset>::OrderPrice,
        VecDeque<<Order as Asset>::OrderId>,
    >,
    bid: BTreeMap<
        <Order as Asset>::OrderPrice,
        VecDeque<<Order as Asset>::OrderId>,
    >,
}

impl<Order: Asset> Default for OrdersByPrice<Order> {
    #[inline]
    fn default() -> Self {
        Self {
            ask: Default::default(),
            bid: Default::default(),
        }
    }
}

impl<Order, S> Index<S> for OrdersByPrice<Order>
where
    Order: Asset<OrderSide = OrderSide>,
    S: Borrow<<Order as Asset>::OrderSide>,
{
    type Output = BTreeMap<
        <Order as Asset>::OrderPrice,
        VecDeque<<Order as Asset>::OrderId>,
    >;

    #[inline]
    fn index(&self, side: S) -> &Self::Output {
        match *side.borrow() {
            OrderSide::Ask => &self.ask,
            OrderSide::Bid => &self.bid,
        }
    }
}

impl<Order, S> IndexMut<S> for OrdersByPrice<Order>
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
