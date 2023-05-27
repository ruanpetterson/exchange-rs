use std::cmp::Reverse;
use std::collections::{BTreeMap, VecDeque};
use std::hash::Hash;
use std::marker::PhantomData;

use compact_str::CompactString;
use indexmap::IndexMap;
use orderbook_core::{Asset, Exchange, ExchangeExt, OrderSide};

pub struct Orderbook<Order: Asset, Trade> {
    #[allow(dead_code)]
    pair: CompactString,
    orders: IndexMap<<Order as Asset>::OrderId, Order>,
    ask: BTreeMap<u64, VecDeque<<Order as Asset>::OrderId>>,
    bid: BTreeMap<Reverse<u64>, VecDeque<<Order as Asset>::OrderId>>,
    _trade: PhantomData<Trade>,
}

impl<Order, Trade> Orderbook<Order, Trade>
where
    Order: Asset,
{
    #[inline]
    pub fn new(pair: &str) -> Self {
        Self {
            pair: CompactString::new_inline(pair),
            orders: IndexMap::new(),
            ask: BTreeMap::new(),
            bid: BTreeMap::new(),
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
    type Order = Order;

    #[inline]
    fn insert(&mut self, order: Self::Order) {
        let level = match order.side() {
            OrderSide::Ask => self
                .ask
                .entry(order.limit_price())
                .or_insert_with(|| VecDeque::with_capacity(8)),
            OrderSide::Bid => self
                .bid
                .entry(Reverse(order.limit_price()))
                .or_insert_with(|| VecDeque::with_capacity(8)),
        };

        level.push_back(order.id());

        self.orders.insert(order.id(), order);
    }

    #[inline]
    fn remove(
        &mut self,
        _order_id: &<Self::Order as Asset>::OrderId,
    ) -> Option<Self::Order> {
        // TODO: implement a way to remove orders. It should not let dangling
        // levels (level with no orders).
        todo!()
    }

    #[inline]
    fn peek(&self, side: &OrderSide) -> Option<&Self::Order> {
        match side {
            OrderSide::Ask => {
                self.ask.first_key_value().map(|(_, level)| level)?
            }
            OrderSide::Bid => {
                self.bid.first_key_value().map(|(_, level)| level)?
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
                self.bid.first_key_value().map(|(_, level)| level)?
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
                // It prevents dagling levels (level with no orders).
                if level.get().len() == 1 {
                    level.remove().pop_front()
                } else {
                    level.get_mut().pop_front()
                }
            }
            OrderSide::Bid => {
                let mut level = self.bid.first_entry()?;
                // It prevents dagling levels (level with no orders).
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
    fn spread(&self) -> Option<(u64, u64)> {
        Some((
            self.peek(&OrderSide::Ask)?.limit_price(),
            self.peek(&OrderSide::Bid)?.limit_price(),
        ))
    }

    fn len(&self) -> (usize, usize) {
        (
            self.ask.iter().fold(0, |acc, (_, level)| acc + level.len()),
            self.bid.iter().fold(0, |acc, (_, level)| acc + level.len()),
        )
    }
}
