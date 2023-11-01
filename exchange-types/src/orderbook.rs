use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, VecDeque};
use std::hash::Hash;
use std::marker::PhantomData;

use either::Either;
use exchange_algo::DefaultExchange;
use exchange_core::{Asset, Exchange, ExchangeExt};
use indexmap::IndexMap;
use num::Zero;

use crate::OrderSide;

pub struct Orderbook<Order: Asset, Trade> {
    orders: IndexMap<<Order as Asset>::OrderId, Order>,
    ask: BTreeMap<
        <Order as Asset>::OrderPrice,
        VecDeque<<Order as Asset>::OrderId>,
    >,
    bid: BTreeMap<
        <Order as Asset>::OrderPrice,
        VecDeque<<Order as Asset>::OrderId>,
    >,
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
            orders: IndexMap::default(),
            ask: BTreeMap::default(),
            bid: BTreeMap::default(),
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
    type Algo = DefaultExchange;
    type Order = Order;

    fn iter(
        &self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> impl Iterator<Item = &Self::Order> {
        let order_id_to_order =
            |order_id: &<Order as Asset>::OrderId| -> &Order {
                self.orders
                    .get(order_id)
                    .expect("every order in tree must algo be in index")
            };

        match side {
            OrderSide::Ask => Either::Left(
                self.ask
                    .values()
                    .flat_map(VecDeque::iter)
                    .map(order_id_to_order),
            ),
            OrderSide::Bid => Either::Right(
                self.bid
                    .values()
                    .rev()
                    .flat_map(VecDeque::iter)
                    .map(order_id_to_order),
            ),
        }
    }

    #[inline]
    fn insert(&mut self, order: Self::Order) {
        let level = match order.side() {
            OrderSide::Ask => self
                .ask
                .entry(
                    order
                        .limit_price()
                        .expect("bookable orders must have a limit price"),
                )
                .or_insert_with(|| VecDeque::with_capacity(8)),
            OrderSide::Bid => self
                .bid
                .entry(
                    order
                        .limit_price()
                        .expect("bookable orders must have a limit price"),
                )
                .or_insert_with(|| VecDeque::with_capacity(8)),
        };

        level.push_back(order.id());

        self.orders.insert(order.id(), order);
    }

    #[inline]
    fn remove(
        &mut self,
        order_id: &<Self::Order as Asset>::OrderId,
    ) -> Option<Self::Order> {
        let order = self.orders.remove(order_id)?;

        let limit_price = order
            .limit_price()
            .expect("bookable orders must have a limit price");

        match order.side() {
            OrderSide::Ask => {
                let Entry::Occupied(mut level) = self.ask.entry(limit_price)
                else {
                    unreachable!();
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
            }
            OrderSide::Bid => {
                let Entry::Occupied(mut level) = self.bid.entry(limit_price)
                else {
                    unreachable!();
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
            }
        }
        .expect("indexed orders must be in the book tree");

        Some(order)
    }

    #[inline]
    fn peek(&self, side: &OrderSide) -> Option<&Self::Order> {
        match side {
            OrderSide::Ask => {
                self.ask.first_key_value().map(|(_, level)| level)?
            }
            OrderSide::Bid => {
                self.bid.last_key_value().map(|(_, level)| level)?
            }
        }
        .front()
        .and_then(|order_id| self.orders.get(order_id))
    }

    #[inline]
    fn peek_mut(&mut self, side: &OrderSide) -> Option<&mut Self::Order> {
        match side {
            OrderSide::Ask => {
                self.ask.first_key_value().map(|(_, level)| level)?
            }
            OrderSide::Bid => {
                self.bid.last_key_value().map(|(_, level)| level)?
            }
        }
        .front()
        .and_then(|order_id| self.orders.get_mut(order_id))
    }

    #[inline]
    fn pop(&mut self, side: &OrderSide) -> Option<Self::Order> {
        match side {
            OrderSide::Ask => {
                let mut level = self.ask.first_entry()?;
                // It prevents dangling levels (level with no orders).
                if level.get().len() == 1 {
                    level.remove().pop_front()
                } else {
                    level.get_mut().pop_front()
                }
            }
            OrderSide::Bid => {
                let mut level = self.bid.first_entry()?;
                // It prevents dangling levels (level with no orders).
                if level.get().len() == 1 {
                    level.remove().pop_front()
                } else {
                    level.get_mut().pop_front()
                }
            }
        }
        .and_then(|order_id| self.orders.remove(&order_id))
    }
}

impl<Order, Trade> ExchangeExt for Orderbook<Order, Trade>
where
    Order: Asset<OrderSide = OrderSide>,
    Order: Asset<Trade = Trade>,
    <Order as Asset>::OrderId: Hash,
{
    fn spread(
        &self,
    ) -> Option<(<Order as Asset>::OrderPrice, <Order as Asset>::OrderPrice)>
    {
        Some((
            self.peek(&OrderSide::Ask)?.limit_price()?,
            self.peek(&OrderSide::Bid)?.limit_price()?,
        ))
    }

    fn len(&self) -> (usize, usize) {
        (
            self.ask.iter().fold(0, |acc, (_, level)| acc + level.len()),
            self.bid.iter().fold(0, |acc, (_, level)| acc + level.len()),
        )
    }

    fn volume(
        &self,
    ) -> (<Order as Asset>::OrderAmount, <Order as Asset>::OrderAmount) {
        let ask = self
            .iter(&OrderSide::Ask)
            .map(|order| order.remaining())
            .reduce(|acc, curr| acc + curr)
            .unwrap_or(Zero::zero());

        let bid = self
            .iter(&OrderSide::Bid)
            .map(|order| order.remaining())
            .reduce(|acc, curr| acc + curr)
            .unwrap_or(Zero::zero());

        (ask, bid)
    }
}
