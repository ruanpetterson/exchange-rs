mod index;

use std::collections::btree_map::Entry;
use std::collections::VecDeque;
use std::hash::Hash;
use std::marker::PhantomData;

use either::Either;
use exchange_core::{Asset, Exchange, ExchangeExt};
use exchange_types::OrderSide;
use num::Zero;

use crate::standard::orderbook::index::{OrdersById, OrdersByPrice};
use crate::standard::MatchingAlgo;

pub struct Orderbook<Order: Asset, Trade> {
    orders_by_id: OrdersById<Order>,
    orders_by_price: OrdersByPrice<Order>,
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
            orders_by_price: Default::default(),
            _trade: PhantomData,
        }
    }
}

impl<Order, Trade> Exchange for Orderbook<Order, Trade>
where
    Order: Asset<OrderSide = OrderSide>,
    Order: Asset<Trade = Trade>,
    <Order as Asset>::OrderId: Hash,
{
    type Algo = MatchingAlgo;
    type Order = Order;

    #[inline]
    fn iter(
        &self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> impl Iterator<Item = &Self::Order> {
        let order_id_to_order =
            |order_id: &<Order as Asset>::OrderId| -> &Order {
                self.orders_by_id
                    .get(order_id)
                    .expect("every order in tree must also be in index")
            };

        match side {
            OrderSide::Ask => Either::Left(
                self.orders_by_price[OrderSide::Ask]
                    .values()
                    .flat_map(VecDeque::iter)
                    .map(order_id_to_order),
            ),
            OrderSide::Bid => Either::Right(
                self.orders_by_price[OrderSide::Bid]
                    .values()
                    .rev()
                    .flat_map(VecDeque::iter)
                    .map(order_id_to_order),
            ),
        }
    }

    #[inline]
    unsafe fn insert(&mut self, order: Self::Order) {
        self.orders_by_price[order.side()]
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
            self.orders_by_price[order.side()].entry(limit_price)
        else {
            unreachable!("orders that lives in index must also be in the tree");
        };

        // It prevents dangling levels (level with no orders).
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
    fn peek(&self, side: &OrderSide) -> Option<&Self::Order> {
        let order_id = match side {
            side @ OrderSide::Ask => {
                self.orders_by_price[side].first_key_value()
            }
            side @ OrderSide::Bid => {
                self.orders_by_price[side].last_key_value()
            }
        }
        .map(|(_, level)| level)?
        .front()?;

        self.orders_by_id
            .get(order_id)
            .expect("every order that lives in tree must also be in the index")
            .into()
    }

    #[inline]
    fn peek_mut(&mut self, side: &OrderSide) -> Option<&mut Self::Order> {
        let order_id = match side {
            side @ OrderSide::Ask => {
                self.orders_by_price[side].first_key_value()
            }
            side @ OrderSide::Bid => {
                self.orders_by_price[side].last_key_value()
            }
        }
        .map(|(_, level)| level)?
        .front()?;

        self.orders_by_id
            .get_mut(order_id)
            .expect("every order that lives in tree must also be in the index")
            .into()
    }

    #[inline]
    fn pop(&mut self, side: &OrderSide) -> Option<Self::Order> {
        let order_id = match side {
            side @ OrderSide::Ask => self.orders_by_price[side].first_entry(),
            side @ OrderSide::Bid => self.orders_by_price[side].last_entry(),
        }
        .and_then(|mut level| {
            // It prevents dangling levels (level with no orders).
            if level.get().len() == 1 {
                level.remove().pop_front()
            } else {
                level.get_mut().pop_front()
            }
        })?;

        self.orders_by_id
            .remove(&order_id)
            .expect("every order that lives in tree must also be in the index")
            .into()
    }
}

impl<Order, Trade> ExchangeExt for Orderbook<Order, Trade>
where
    Order: Asset<OrderSide = OrderSide>,
    Order: Asset<Trade = Trade>,
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
            self.orders_by_price[OrderSide::Ask]
                .iter()
                .fold(0, |acc, (_, level)| acc + level.len()),
            self.orders_by_price[OrderSide::Bid]
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
