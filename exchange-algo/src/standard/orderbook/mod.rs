mod index;

use std::collections::btree_map::Entry;
use std::hash::Hash;
use std::marker::PhantomData;

use exchange_core::{Asset, Engine, EngineExt, Exchange, Tree};
use exchange_types::OrderSide;
use num::Zero;

use crate::standard::orderbook::index::{OrdersById, OrdersBySide};
use crate::standard::MatchingAlgo;

pub struct Orderbook<Order: Asset, Trade> {
    orders_by_id: OrdersById<Order>,
    orders_by_side: OrdersBySide<Order>,
    _trade: PhantomData<Trade>,
}

impl<Order, Trade> Orderbook<Order, Trade>
where
    Order: Asset,
{
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Order, Trade> Default for Orderbook<Order, Trade>
where
    Order: Asset,
{
    #[inline]
    fn default() -> Self {
        Self {
            orders_by_id: Default::default(),
            orders_by_side: Default::default(),
            _trade: PhantomData,
        }
    }
}

impl<Order, Trade> Engine for Orderbook<Order, Trade>
where
    Order: Asset<OrderSide = OrderSide>,
    Order: Asset<Trade = Trade>,
    Order: Exchange,
{
    type Algo = MatchingAlgo;
}

impl<Order, Trade> Tree for Orderbook<Order, Trade>
where
    Order: Asset<OrderSide = OrderSide>,
    Order: Asset<Trade = Trade>,
    Order: Exchange,
{
    type Order = Order;
    type OrderRef<'e> = &'e Order where Self: 'e;
    type OrderRefMut<'e> = &'e mut Order where Self: 'e;

    #[inline]
    fn iter(
        &self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> impl Iterator<Item = Self::OrderRef<'_>> + '_ {
        let order_id_to_order =
            |order_id: &<Order as Asset>::OrderId| -> &Order {
                self.orders_by_id
                    .get(order_id)
                    .expect("every order in tree must also be in index")
            };

        self.orders_by_side.iter(side).map(order_id_to_order)
    }

    #[inline]
    unsafe fn insert(&mut self, order: Self::Order) {
        self.orders_by_side[order.side()]
            .entry(
                order
                    .limit_price()
                    .expect("bookable orders must have a limit price"),
            )
            .or_default()
            .push_back(order.id());

        self.orders_by_id.insert(order.id(), order);
    }

    #[inline]
    fn remove(
        &mut self,
        order_id: &<Self::Order as Asset>::OrderId,
    ) -> Option<Self::Order> {
        let order = self.orders_by_id.remove(order_id)?;

        let limit_price = order
            .limit_price()
            .expect("bookable orders must have a limit price");

        let Entry::Occupied(mut level) =
            self.orders_by_side[order.side()].entry(limit_price)
        else {
            unreachable!("orders that lives in index must also be in the tree");
        };

        // This prevents dangling levels (level with no orders).
        if level.get().len() == 1 {
            level.remove().pop_front()
        } else {
            level
                .get()
                .iter()
                .position(|&order_id| order.id() == order_id)
                .and_then(|index| level.get_mut().remove(index))
        }
        .expect("indexed orders must be in the book tree");

        assert!(
            &order.id() == order_id,
            "order id must be the same; something is wrong otherwise"
        );

        order.into()
    }

    #[inline]
    fn peek(&self, side: &OrderSide) -> Option<Self::OrderRef<'_>> {
        let order_id = self.orders_by_side.peek(side)?;

        self.orders_by_id
            .get(order_id)
            .expect("every order that lives in tree must also be in the index")
            .into()
    }

    #[inline]
    fn peek_mut(&mut self, side: &OrderSide) -> Option<Self::OrderRefMut<'_>> {
        let order_id = self.orders_by_side.peek(side)?;

        self.orders_by_id
            .get_mut(order_id)
            .expect("every order that lives in tree must also be in the index")
            .into()
    }

    #[inline]
    fn pop(&mut self, side: &OrderSide) -> Option<Self::Order> {
        let mut level = match side {
            side @ OrderSide::Ask => self.orders_by_side[side].first_entry(),
            side @ OrderSide::Bid => self.orders_by_side[side].last_entry(),
        }?;

        let order_id = if level.get().len() == 1 {
            // This prevents dangling levels (level with no orders).
            level.remove().pop_front()
        } else {
            level.get_mut().pop_front()
        }
        .expect("level should always have an order");

        self.orders_by_id
            .remove(&order_id)
            .expect("every order that lives in tree must also be in the index")
            .into()
    }
}

impl<Order, Trade> EngineExt for Orderbook<Order, Trade>
where
    Order: Asset<OrderSide = OrderSide>,
    Order: Asset<Trade = Trade>,
    Order: Exchange,
    <Order as Asset>::OrderId: Hash,
{
    #[inline]
    fn spread(
        &self,
    ) -> Option<(<Order as Asset>::OrderPrice, <Order as Asset>::OrderPrice)>
    {
        Some((
            self.peek(&OrderSide::Ask)?.limit_price()?,
            self.peek(&OrderSide::Bid)?.limit_price()?,
        ))
    }

    #[inline]
    fn len(&self) -> (usize, usize) {
        (
            self.orders_by_side[OrderSide::Ask]
                .iter()
                .fold(0, |acc, (_, level)| acc + level.len()),
            self.orders_by_side[OrderSide::Bid]
                .iter()
                .fold(0, |acc, (_, level)| acc + level.len()),
        )
    }

    #[inline]
    fn volume(
        &self,
    ) -> (<Order as Asset>::OrderAmount, <Order as Asset>::OrderAmount) {
        let ask = self
            .iter(&OrderSide::Ask)
            .map(Order::remaining)
            .reduce(|acc, curr| acc + curr)
            .unwrap_or_else(Zero::zero);

        let bid = self
            .iter(&OrderSide::Bid)
            .map(Order::remaining)
            .reduce(|acc, curr| acc + curr)
            .unwrap_or_else(Zero::zero);

        (ask, bid)
    }
}

#[cfg(any(test, feature = "test"))]
#[doc(hidden)]
pub(crate) mod __fmt {
    use std::fmt;

    use super::*;

    #[cfg(any(test, feature = "test"))]
    impl<Order, Trade> fmt::Debug for Orderbook<Order, Trade>
    where
        Order: Asset<Trade = Trade>,
        Order: Asset<OrderSide = OrderSide>,
        Order: Exchange,
        <Order as Asset>::OrderAmount: fmt::Debug,
        <Order as Asset>::OrderPrice: fmt::Debug,
        <Order as Asset>::OrderStatus: fmt::Debug,
    {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            __fmt::OrderbookView::from(self).fmt(f)
        }
    }

    #[repr(transparent)]
    pub struct OrderbookView<'a, Order: Asset, Trade>(
        &'a Orderbook<Order, Trade>,
    );

    impl<'a, Order: Asset, Trade> OrderbookView<'a, Order, Trade> {
        #[inline]
        pub const fn new(orderbook: &'a Orderbook<Order, Trade>) -> Self {
            Self(orderbook)
        }
    }

    impl<'a, Order: Asset, Trade> From<&'a Orderbook<Order, Trade>>
        for OrderbookView<'a, Order, Trade>
    {
        #[inline]
        fn from(orderbook: &'a Orderbook<Order, Trade>) -> Self {
            Self::new(orderbook)
        }
    }

    impl<'a, Order, Trade> fmt::Debug for OrderbookView<'a, Order, Trade>
    where
        Order: Asset<Trade = Trade>,
        Order: Asset<OrderSide = OrderSide>,
        Order: Exchange,
        <Order as Asset>::OrderAmount: fmt::Debug,
        <Order as Asset>::OrderPrice: fmt::Debug,
        <Order as Asset>::OrderSide: fmt::Debug,
        <Order as Asset>::OrderStatus: fmt::Debug,
    {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let filter = |ref side| {
                self.0
                    .iter(side)
                    .map(OrderbookOrderView)
                    .collect::<Vec<_>>()
            };

            f.debug_map()
                .entry(&OrderSide::Ask, &filter(OrderSide::Ask))
                .entry(&OrderSide::Bid, &filter(OrderSide::Bid))
                .finish()
        }
    }

    #[repr(transparent)]
    struct OrderbookOrderView<'o, Order>(&'o Order);
    impl<'o, Order: Asset> fmt::Debug for OrderbookOrderView<'o, Order>
    where
        <Order as Asset>::OrderAmount: fmt::Debug,
        <Order as Asset>::OrderPrice: fmt::Debug,
        <Order as Asset>::OrderStatus: fmt::Debug,
    {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Order")
                .field(
                    "limit_price",
                    &self
                        .0
                        .limit_price()
                        .expect("orderbook orders always have limit price"),
                )
                .field("remaining", &self.0.remaining())
                .field("status", &self.0.status())
                .finish()
        }
    }
}
