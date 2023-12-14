use std::borrow::Borrow;
use std::cmp::{Ordering, Reverse};
use std::ops::AddAssign;

use exchange_core::Asset;
use thiserror::Error;

use crate::order_type::TimeInForce;
use crate::trade::{PriceError, SideError, StatusError, TradeError};
use crate::{OrderId, OrderSide, OrderStatus, OrderType, Trade};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Order {
    id: OrderId,
    side: OrderSide,
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    type_: OrderType,
    status: OrderStatus,
}

impl Order {
    #[inline]
    pub fn new(id: OrderId, side: OrderSide, type_: OrderType) -> Self {
        Self {
            id,
            side,
            type_,
            status: OrderStatus::Open,
        }
    }

    #[inline]
    #[cfg(any(test, feature = "test"))]
    pub fn builder() -> builder::Builder<(), ()> {
        builder::Builder::new()
    }

    #[inline]
    #[deprecated]
    pub fn new_limit(
        id: OrderId,
        side: OrderSide,
        limit_price: u64,
        amount: u64,
    ) -> Self {
        Self {
            id,
            side,
            type_: OrderType::Limit {
                limit_price,
                time_in_force: Default::default(),
                amount,
                filled: 0,
            },
            status: OrderStatus::Open,
        }
    }

    /// Fill an order within the specified amount.
    ///
    /// # Panics
    ///
    /// Panics if `amount` is greater then `remaining`.
    #[inline]
    pub(crate) fn fill(&mut self, amount: u64) {
        self.try_fill(amount)
            .expect("order does not have available amount to fill")
    }

    /// Fill an order within the specified amount.
    ///
    /// # Safety
    ///
    /// This results in an unreliable state when current `Order::filled`
    /// overflows `Order::amount` or given amount is zero.
    #[inline]
    pub(crate) unsafe fn fill_unchecked(&mut self, amount: u64) {
        let filled = match self.type_ {
            OrderType::Limit { ref mut filled, .. }
            | OrderType::Market { ref mut filled, .. } => filled,
        };

        filled.add_assign(amount);

        self.status = if self.remaining() == 0 {
            OrderStatus::Completed
        } else {
            OrderStatus::Partial
        };
    }

    /// Fill an order within the specified amount, returning an error if
    /// something fails.
    #[inline]
    pub(crate) fn try_fill(&mut self, amount: u64) -> Result<(), OrderError> {
        if amount == 0 {
            return Err(OrderError::NoFill);
        }

        if amount > self.remaining() {
            return Err(OrderError::Overfill);
        }

        // SAFETY: we already guarantee that `remaining >= amount > 0`.
        unsafe { self.fill_unchecked(amount) };

        Ok(())
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
        self.limit_price().partial_cmp(&other.limit_price())
    }
}

impl Asset for Order {
    type OrderAmount = u64;
    type OrderId = OrderId;
    type OrderPrice = u64;
    type OrderSide = OrderSide;
    type OrderStatus = OrderStatus;
    type Trade = Trade;
    type TradeError = TradeError;

    #[inline]
    fn id(&self) -> OrderId {
        self.id
    }

    #[inline]
    fn side(&self) -> OrderSide {
        self.side
    }

    #[inline]
    fn limit_price(&self) -> Option<Self::OrderPrice> {
        match self.type_ {
            OrderType::Limit { limit_price, .. } => Some(limit_price),
            _ => None,
        }
    }

    #[inline]
    fn remaining(&self) -> Self::OrderAmount {
        match self.type_ {
            OrderType::Limit { amount, filled, .. }
            | OrderType::Market { amount, filled, .. } => amount - filled,
        }
    }

    #[inline]
    fn status(&self) -> OrderStatus {
        self.status
    }

    #[inline]
    fn is_fill_or_kill(&self) -> bool {
        match self.type_ {
            OrderType::Market { all_or_none, .. }
            | OrderType::Limit {
                time_in_force: TimeInForce::ImmediateOrCancel { all_or_none },
                ..
            } => all_or_none,
            _ => false,
        }
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
    fn is_immediate_or_cancel(&self) -> bool {
        matches!(
            self.type_,
            OrderType::Limit {
                time_in_force: TimeInForce::ImmediateOrCancel { .. },
                ..
            } | OrderType::Market { .. }
        )
    }

    #[inline]
    fn is_post_only(&self) -> bool {
        matches!(self.type_, OrderType::Limit { time_in_force: TimeInForce::GoodTillCancel { post_only }, .. } if post_only)
    }

    #[inline]
    fn trade(
        &mut self,
        other: &mut Self,
    ) -> Result<Self::Trade, Self::TradeError> {
        let (taker, maker) = (self, other);

        Trade::new(taker, maker)
    }

    #[inline]
    fn matches(&self, other: &Self) -> Result<(), Self::TradeError> {
        let (taker, maker) = (self, other);

        // Matching cannot occur between closed orders.
        if taker.is_closed() || maker.is_closed() {
            return Err(StatusError::Closed)?;
        }

        if taker.side() == maker.side() {
            return Err(SideError::Conflict)?;
        }

        let maker_limit_price = maker
            .limit_price()
            .expect("market makers always have a limit price");

        if let Some(taker_limit_price) = taker.limit_price() {
            match (taker.side(), maker.side()) {
                (OrderSide::Ask, OrderSide::Bid)
                    if taker_limit_price > maker_limit_price =>
                {
                    return Err(PriceError::Incompatible)?;
                }
                (OrderSide::Bid, OrderSide::Ask)
                    if taker_limit_price < maker_limit_price =>
                {
                    return Err(PriceError::Incompatible)?;
                }
                _ => (),
            }
        };

        Ok(())
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

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("filling amount exceeds remaining amount")]
    Overfill,
    #[error("empty filling is not allowed")]
    NoFill,
}

#[cfg(any(test, feature = "test"))]
mod builder {
    use std::hint::unreachable_unchecked;
    use std::marker::PhantomData;
    use std::mem::MaybeUninit;

    use super::*;

    pub struct Builder<S, T> {
        side: S,
        type_: MaybeUninit<OrderType>,
        type_variant: PhantomData<T>,
    }

    pub struct Limit<T>(PhantomData<T>);
    pub struct Market {}

    pub trait TypeVariant {}
    impl<T: LimitTypeVariant> TypeVariant for Limit<T> {}
    impl TypeVariant for Market {}

    pub enum GoodTillCancel {}
    pub enum ImmediateOrCancel {}

    pub trait LimitTypeVariant {}
    impl LimitTypeVariant for GoodTillCancel {}
    impl LimitTypeVariant for ImmediateOrCancel {}

    impl Builder<(), ()> {
        #[inline]
        pub const fn new() -> Self {
            Self {
                side: (),
                type_: MaybeUninit::uninit(),
                type_variant: PhantomData,
            }
        }
    }

    impl<S, T> Builder<S, T> {
        #[inline]
        pub const fn side(&self, side: OrderSide) -> Builder<OrderSide, T> {
            Builder {
                side,
                type_: self.type_,
                type_variant: self.type_variant,
            }
        }
    }

    impl<T> Builder<OrderSide, T> {
        #[inline]
        pub const fn limit(
            &self,
            limit_price: u64,
            amount: u64,
        ) -> Builder<OrderSide, Limit<GoodTillCancel>> {
            let type_ = OrderType::Limit {
                limit_price,
                time_in_force: TimeInForce::GoodTillCancel { post_only: false },
                amount,
                filled: 0,
            };

            Builder {
                side: self.side,
                type_: MaybeUninit::new(type_),
                type_variant: PhantomData,
            }
        }

        #[inline]
        pub const fn market(&self, amount: u64) -> Builder<OrderSide, Market> {
            let type_ = OrderType::Market {
                all_or_none: false,
                amount,
                filled: 0,
            };

            Builder {
                side: self.side,
                type_: MaybeUninit::new(type_),
                type_variant: PhantomData,
            }
        }
    }

    impl<T: LimitTypeVariant> Builder<OrderSide, Limit<T>> {
        #[inline]
        pub const fn gtc(&self) -> Builder<OrderSide, Limit<GoodTillCancel>> {
            let OrderType::Limit {
                limit_price,
                time_in_force: _,
                amount,
                filled,
            } = self.type_()
            else {
                // SAFETY: since this is a `Builder<_, Limit<_>>`, this will
                // always be `Limit`.
                unsafe { unreachable_unchecked() }
            };

            let type_ = OrderType::Limit {
                limit_price,
                time_in_force: TimeInForce::GoodTillCancel { post_only: false },
                amount,
                filled,
            };

            Builder {
                side: self.side,
                type_: MaybeUninit::new(type_),
                type_variant: PhantomData,
            }
        }

        #[inline]
        pub const fn ioc(
            &self,
        ) -> Builder<OrderSide, Limit<ImmediateOrCancel>> {
            let OrderType::Limit {
                limit_price,
                time_in_force: _,
                amount,
                filled,
            } = self.type_()
            else {
                // SAFETY: since this is a `Builder<_, Limit<_>>`, this will
                // always be `Limit`.
                unsafe { unreachable_unchecked() }
            };

            let type_ = OrderType::Limit {
                limit_price,
                time_in_force: TimeInForce::ImmediateOrCancel {
                    all_or_none: false,
                },
                amount,
                filled,
            };

            Builder {
                side: self.side,
                type_: MaybeUninit::new(type_),
                type_variant: PhantomData,
            }
        }
    }

    impl Builder<OrderSide, Limit<GoodTillCancel>> {
        #[inline]
        pub const fn post_only(
            &self,
        ) -> Builder<OrderSide, Limit<GoodTillCancel>> {
            let OrderType::Limit {
                limit_price,
                time_in_force: _,
                amount,
                filled,
            } = self.type_()
            else {
                // SAFETY: since this is a `Builder<_, Limit<_>>`, this will
                // always be `Limit`.
                unsafe { unreachable_unchecked() }
            };

            let type_ = OrderType::Limit {
                limit_price,
                time_in_force: TimeInForce::GoodTillCancel { post_only: true },
                amount,
                filled,
            };

            Builder {
                side: self.side,
                type_: MaybeUninit::new(type_),
                type_variant: PhantomData,
            }
        }
    }

    impl Builder<OrderSide, Limit<ImmediateOrCancel>> {
        #[inline]
        pub const fn all_or_none(
            &self,
        ) -> Builder<OrderSide, Limit<ImmediateOrCancel>> {
            let OrderType::Limit {
                limit_price,
                time_in_force: _,
                amount,
                filled,
            } = self.type_()
            else {
                // SAFETY: since this is a `Builder<_, Limit<_>>`, this will
                // always be `Limit`.
                unsafe { unreachable_unchecked() }
            };

            let type_ = OrderType::Limit {
                limit_price,
                time_in_force: TimeInForce::ImmediateOrCancel {
                    all_or_none: true,
                },
                amount,
                filled,
            };

            Builder {
                side: self.side,
                type_: MaybeUninit::new(type_),
                type_variant: PhantomData,
            }
        }
    }

    impl Builder<OrderSide, Market> {
        #[inline]
        pub const fn all_or_none(&self) -> Builder<OrderSide, Market> {
            let OrderType::Market {
                all_or_none: _,
                amount,
                filled,
            } = self.type_()
            else {
                // SAFETY: since this is a `Builder<_, Market<_>>`, this will
                // always be `Market`.
                unsafe { unreachable_unchecked() }
            };

            let type_ = OrderType::Market {
                all_or_none: true,
                amount,
                filled,
            };

            Builder {
                side: self.side,
                type_: MaybeUninit::new(type_),
                type_variant: PhantomData,
            }
        }
    }

    impl<T: TypeVariant> Builder<OrderSide, T> {
        #[inline]
        const fn type_(&self) -> OrderType {
            // SAFETY: whenever a `T` that implements `TypeVariant` is set, the
            // `Builder::type_` is initialized.
            unsafe { self.type_.assume_init() }
        }

        #[inline]
        pub fn build(self) -> Order {
            Order {
                id: OrderId::random(),
                side: self.side,
                type_: self.type_(),
                status: OrderStatus::Open,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod valid_trades {
        use super::*;

        #[test]
        fn same_prices() {
            let mut ask =
                Order::builder().side(OrderSide::Ask).limit(10, 10).build();
            let mut bid =
                Order::builder().side(OrderSide::Bid).limit(10, 10).build();

            assert!(ask.trade(&mut bid).is_ok());
        }

        #[test]
        fn different_prices() {
            let mut ask =
                Order::builder().side(OrderSide::Ask).limit(10, 10).build();
            let mut bid =
                Order::builder().side(OrderSide::Bid).limit(20, 10).build();

            assert!(ask.trade(&mut bid).is_ok());
        }

        #[test]
        fn partial_maker() {
            let mut ask =
                Order::builder().side(OrderSide::Ask).limit(10, 5).build();
            let mut bid =
                Order::builder().side(OrderSide::Bid).limit(20, 10).build();

            assert!(ask.trade(&mut bid).is_ok());
            assert!(ask.is_closed());
            assert!(!bid.is_closed());
        }

        #[test]
        fn partial_taker() {
            let mut ask =
                Order::builder().side(OrderSide::Ask).limit(10, 10).build();
            let mut bid =
                Order::builder().side(OrderSide::Bid).limit(20, 5).build();

            assert!(ask.trade(&mut bid).is_ok());
            assert!(!ask.is_closed());
            assert!(bid.is_closed());
        }
    }

    mod invalid_trades {
        use super::*;

        #[test]
        fn same_side() {
            let mut ask_1 =
                Order::builder().side(OrderSide::Ask).limit(10, 10).build();
            let mut ask_2 =
                Order::builder().side(OrderSide::Ask).limit(10, 10).build();

            assert!(ask_1.trade(&mut ask_2).is_err());
        }

        #[test]
        fn incompatible_prices() {
            let mut ask =
                Order::builder().side(OrderSide::Ask).limit(20, 10).build();
            let mut bid =
                Order::builder().side(OrderSide::Bid).limit(10, 10).build();

            assert!(ask.trade(&mut bid).is_err());
        }
    }

    #[test]
    fn cancel_order() {
        let mut ask =
            Order::builder().side(OrderSide::Ask).limit(10, 10).build();
        ask.cancel();
        assert_eq!(ask.status(), OrderStatus::Cancelled);
    }

    #[test]
    fn close_order() {
        let mut ask =
            Order::builder().side(OrderSide::Ask).limit(10, 10).build();
        let mut bid =
            Order::builder().side(OrderSide::Bid).limit(10, 5).build();

        assert!(ask.trade(&mut bid).is_ok());

        ask.cancel();

        assert_eq!(ask.status(), OrderStatus::Closed);
    }
}
