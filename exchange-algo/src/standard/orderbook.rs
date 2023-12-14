use std::collections::btree_map::Entry;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use exchange_core::{Asset, Exchange, ExchangeExt};
use exchange_types::{Order, OrderSide, Trade};
use num::Zero;
use tap::Pipe as _;

use crate::standard::index::{OrdersById, OrdersBySide};
use crate::standard::{Error, MatchingAlgo};

pub struct Orderbook<Order: Asset, Trade> {
    orders_by_id: OrdersById,
    orders_by_side: OrdersBySide<Order>,
    _trade: PhantomData<Trade>,
}

pub struct OrderRef<'e> {
    order: Order,
    marker: PhantomData<&'e ()>,
}

impl<'e> Deref for OrderRef<'e> {
    type Target = Order;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.order
    }
}

pub struct OrderRefMut<'e> {
    order: Order,
    exchange: &'e mut Orderbook<Order, Trade>,
}

impl<'e> Deref for OrderRefMut<'e> {
    type Target = Order;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.order
    }
}

impl<'e> DerefMut for OrderRefMut<'e> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.order
    }
}

impl<'e> Drop for OrderRefMut<'e> {
    #[inline]
    fn drop(&mut self) {
        // Update data on disk.
        self.exchange.orders_by_id.insert(self.id(), &self.order);

        if self.is_closed() {
            let removed_order = self
                .exchange
                .remove(&self.id())
                .expect("booked order must always be in the exchange");

            assert!(self.id() == removed_order.id());
        }
    }
}

impl Default for Orderbook<Order, Trade> {
    fn default() -> Self {
        Self::new()
    }
}

impl Orderbook<Order, Trade> {
    #[inline]
    pub fn new() -> Self {
        Self {
            orders_by_id: Default::default(),
            orders_by_side: Default::default(),
            _trade: PhantomData,
        }
    }

    #[inline]
    pub fn use_sled(path: impl AsRef<Path>) -> Result<Self, Error> {
        let orders_by_id = OrdersById::use_sled(path)?;
        let mut orders_by_side: OrdersBySide<_> = Default::default();

        for order in orders_by_id.iter() {
            orders_by_side[order.side()]
                .entry(
                    order
                        .limit_price()
                        .expect("bookable orders must have a limit price"),
                )
                .or_default()
                .push_back(order.id());
        }

        Self {
            orders_by_id,
            orders_by_side,
            _trade: PhantomData,
        }
        .pipe(Ok)
    }
}

impl Exchange for Orderbook<Order, Trade> {
    type Algo = MatchingAlgo;
    type Order = Order;
    type OrderRef<'e> = OrderRef<'e> where Self: 'e;
    type OrderRefMut<'e> = OrderRefMut<'e> where Self: 'e;

    #[inline]
    fn iter(
        &self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> impl Iterator<Item = Self::OrderRef<'_>> + '_ {
        let order_id_to_order =
            |order_id: &<Order as Asset>::OrderId| -> Self::OrderRef<'_> {
                self.orders_by_id
                    .get(order_id)
                    .map(|order| OrderRef {
                        order,
                        marker: PhantomData,
                    })
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

        self.orders_by_id.insert(order.id(), &order);
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
        let order = self
            .orders_by_id
            .get(order_id)
            .map(|order| OrderRef {
                order,
                marker: PhantomData,
            })
            .expect("every order that lives in tree must also be in the index");

        Some(order)
    }

    #[inline]
    fn peek_mut(&mut self, side: &OrderSide) -> Option<Self::OrderRefMut<'_>> {
        let order_id = self.orders_by_side.peek(side)?;

        let order = self
            .orders_by_id
            .get(order_id)
            .expect("every order that lives in tree must also be in the index");

        OrderRefMut {
            order,
            exchange: self,
        }
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

impl ExchangeExt for Orderbook<Order, Trade> {
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
            .map(|order| order.remaining())
            .reduce(|acc, curr| acc + curr)
            .unwrap_or_else(Zero::zero);

        let bid = self
            .iter(&OrderSide::Bid)
            .map(|order| order.remaining())
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
    impl fmt::Debug for Orderbook<Order, Trade> {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            __fmt::OrderbookView::from(self).fmt(f)
        }
    }

    #[repr(transparent)]
    pub struct OrderbookView<'a>(&'a Orderbook<Order, Trade>);

    impl<'a> OrderbookView<'a> {
        #[inline]
        pub const fn new(orderbook: &'a Orderbook<Order, Trade>) -> Self {
            Self(orderbook)
        }
    }

    impl<'a> From<&'a Orderbook<Order, Trade>> for OrderbookView<'a> {
        #[inline]
        fn from(orderbook: &'a Orderbook<Order, Trade>) -> Self {
            Self::new(orderbook)
        }
    }

    impl<'a> fmt::Debug for OrderbookView<'a> {
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
    struct OrderbookOrderView<'o>(OrderRef<'o>);
    impl<'o> fmt::Debug for OrderbookOrderView<'o> {
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
