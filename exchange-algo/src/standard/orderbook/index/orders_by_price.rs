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

impl<Order> Index<<Order as Asset>::OrderSide> for OrdersByPrice<Order>
where
    Order: Asset<OrderSide = OrderSide>,
{
    type Output = BTreeMap<
        <Order as Asset>::OrderPrice,
        VecDeque<<Order as Asset>::OrderId>,
    >;

    #[inline]
    fn index(&self, side: <Order as Asset>::OrderSide) -> &Self::Output {
        match side {
            OrderSide::Ask => &self.ask,
            OrderSide::Bid => &self.bid,
        }
    }
}

impl<Order> IndexMut<<Order as Asset>::OrderSide> for OrdersByPrice<Order>
where
    Order: Asset<OrderSide = OrderSide>,
{
    #[inline]
    fn index_mut(
        &mut self,
        side: <Order as Asset>::OrderSide,
    ) -> &mut Self::Output {
        match side {
            OrderSide::Ask => &mut self.ask,
            OrderSide::Bid => &mut self.bid,
        }
    }
}
