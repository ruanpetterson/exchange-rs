use std::borrow::Borrow;
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::ops::AddAssign;

use either::Either;
use exchange_core::Asset;
use rust_decimal::Decimal;

use crate::error::OrderError;
use crate::error::TradeError;
use crate::order_type::ByBase;
use crate::order_type::ByFunds;
use crate::order_type::PricedBy;
use crate::order_type::TimeInForce;
use crate::Notional;
use crate::OrderId;
use crate::OrderSide;
use crate::OrderStatus;
use crate::OrderType;
use crate::Price;
use crate::Quantity;
use crate::Trade;

mod limit;
pub use limit::LimitOrder;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Order {
    id: OrderId,
    side: OrderSide,
    #[cfg_attr(feature = "serde", serde(flatten))]
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
        limit_price: impl Into<Price>,
        quantity: impl Into<Quantity>,
    ) -> Self {
        Self {
            id,
            side,
            type_: OrderType::Limit {
                limit_price: limit_price.into(),
                time_in_force: Default::default(),
                priced_by: ByBase {
                    quantity: quantity.into(),
                    filled: Decimal::ZERO.into(),
                },
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
    #[track_caller]
    pub(crate) fn fill(&mut self, quantity: Quantity, price: Price) {
        self.try_fill(quantity, price)
            .expect("order does not have available amount to fill")
    }

    /// Fill an order within the specified amount.
    ///
    /// # Safety
    ///
    /// This results in an unreliable state when current `Order::filled`
    /// overflows `Order::amount` or given amount is zero.
    #[inline]
    pub(crate) unsafe fn fill_unchecked(
        &mut self,
        quantity: Quantity,
        price: Price,
    ) {
        match self.type_ {
            OrderType::Limit {
                priced_by: ByBase { ref mut filled, .. },
                ..
            }
            | OrderType::Market {
                priced_by: PricedBy::Base(ByBase { ref mut filled, .. }),
                ..
            } => filled.add_assign(quantity),
            OrderType::Market {
                priced_by: PricedBy::Funds(ByFunds { ref mut filled, .. }),
                ..
            } => filled.add_assign(quantity * price),
        };

        self.status = if match self.remaining() {
            Either::Left(notional) => notional.is_zero(),
            Either::Right(quantity) => quantity.is_zero(),
        } {
            OrderStatus::Completed
        } else {
            OrderStatus::Partial
        };
    }

    /// Fill an order within the specified amount, returning an error if
    /// something fails.
    #[inline]
    pub(crate) fn try_fill(
        &mut self,
        quantity: Quantity,
        price: Price,
    ) -> Result<(), OrderError> {
        if quantity.is_zero() {
            return Err(OrderError::NoFill);
        }

        match self.remaining() {
            Either::Left(notional) => {
                if quantity * price > notional {
                    return Err(OrderError::Overfill);
                }
            }
            Either::Right(remaining) => {
                if quantity > remaining {
                    return Err(OrderError::Overfill);
                }
            }
        }

        // SAFETY: we already guarantee that `remaining >= quantity > 0`.
        unsafe { self.fill_unchecked(quantity, price) };

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
    type OrderId = OrderId;
    type OrderNotional = Notional;
    type OrderPrice = Price;
    type OrderQuantity = Quantity;
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
    fn remaining(&self) -> Either<Self::OrderNotional, Self::OrderQuantity> {
        match self.type_ {
            OrderType::Limit {
                priced_by: ByBase { quantity, filled },
                ..
            }
            | OrderType::Market {
                priced_by: PricedBy::Base(ByBase { quantity, filled }),
                ..
            } => Either::Right(quantity - filled),
            OrderType::Market {
                priced_by: PricedBy::Funds(ByFunds { funds, filled }),
                ..
            } => Either::Left(funds - filled),
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
    fn is_open(&self) -> bool {
        !self.is_closed()
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
    fn cancel(&mut self) {
        match self.status() {
            OrderStatus::Open => self.status = OrderStatus::Cancelled,
            OrderStatus::Partial => self.status = OrderStatus::Closed,
            _ => (),
        }
    }
}

#[cfg(any(test, feature = "test"))]
mod builder {
    use std::hint::unreachable_unchecked;
    use std::marker::PhantomData;
    use std::mem::MaybeUninit;

    use super::*;
    use crate::order_type::PricedBy;

    enum Uninhabited {}

    pub struct Builder<S, T> {
        side: S,
        type_: MaybeUninit<OrderType>,
        type_variant: PhantomData<T>,
    }

    pub struct Limit<T>(Uninhabited, PhantomData<T>);
    pub struct Market(Uninhabited);

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
        pub fn limit(
            &self,
            limit_price: impl Into<Price>,
            quantity: impl Into<Quantity>,
        ) -> Builder<OrderSide, Limit<GoodTillCancel>> {
            let type_ = OrderType::Limit {
                limit_price: limit_price.into(),
                time_in_force: TimeInForce::GoodTillCancel { post_only: false },
                priced_by: ByBase {
                    quantity: quantity.into(),
                    filled: Decimal::ZERO.into(),
                },
            };

            Builder {
                side: self.side,
                type_: MaybeUninit::new(type_),
                type_variant: PhantomData,
            }
        }

        #[inline]
        pub fn market(
            &self,
            quantity: impl Into<Quantity>,
        ) -> Builder<OrderSide, Market> {
            let type_ = OrderType::Market {
                all_or_none: false,
                priced_by: PricedBy::Base(ByBase {
                    quantity: quantity.into(),
                    filled: Decimal::ZERO.into(),
                }),
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
                priced_by,
            } = self.type_()
            else {
                // SAFETY: since this is a `Builder<_, Limit<_>>`, this will
                // always be `Limit`.
                unsafe { unreachable_unchecked() }
            };

            let type_ = OrderType::Limit {
                limit_price,
                time_in_force: TimeInForce::GoodTillCancel { post_only: false },
                priced_by,
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
                priced_by,
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
                priced_by,
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
                priced_by,
            } = self.type_()
            else {
                // SAFETY: since this is a `Builder<_, Limit<_>>`, this will
                // always be `Limit`.
                unsafe { unreachable_unchecked() }
            };

            let type_ = OrderType::Limit {
                limit_price,
                time_in_force: TimeInForce::GoodTillCancel { post_only: true },
                priced_by,
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
                priced_by,
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
                priced_by,
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
                priced_by,
            } = self.type_()
            else {
                // SAFETY: since this is a `Builder<_, Market<_>>`, this will
                // always be `Market`.
                unsafe { unreachable_unchecked() }
            };

            let type_ = OrderType::Market {
                all_or_none: true,
                priced_by,
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
    use exchange_core::Trade as _;
    use rust_decimal_macros::dec;

    use super::*;

    mod valid_trades {
        use super::*;

        #[test]
        fn same_prices() {
            let mut ask: LimitOrder = Order::builder()
                .side(OrderSide::Ask)
                .limit(dec!(10), dec!(10))
                .build()
                .try_into()
                .unwrap();
            let mut bid = Order::builder()
                .side(OrderSide::Bid)
                .limit(dec!(10), dec!(10))
                .build();

            assert!(ask.trade(&mut bid).is_ok());
        }

        #[test]
        fn different_prices() {
            let mut ask: LimitOrder = Order::builder()
                .side(OrderSide::Ask)
                .limit(dec!(10), dec!(10))
                .build()
                .try_into()
                .unwrap();
            let mut bid = Order::builder()
                .side(OrderSide::Bid)
                .limit(dec!(20), dec!(10))
                .build();

            assert!(ask.trade(&mut bid).is_ok());
        }

        #[test]
        fn partial_maker() {
            let mut ask: LimitOrder = Order::builder()
                .side(OrderSide::Ask)
                .limit(dec!(10), dec!(5))
                .build()
                .try_into()
                .unwrap();
            let mut bid = Order::builder()
                .side(OrderSide::Bid)
                .limit(dec!(20), dec!(10))
                .build();

            assert!(ask.trade(&mut bid).is_ok());
            assert!(ask.is_closed());
            assert!(!bid.is_closed());
        }

        #[test]
        fn partial_taker() {
            let mut ask: LimitOrder = Order::builder()
                .side(OrderSide::Ask)
                .limit(dec!(10), dec!(10))
                .build()
                .try_into()
                .unwrap();
            let mut bid = Order::builder()
                .side(OrderSide::Bid)
                .limit(dec!(20), dec!(5))
                .build();

            assert!(ask.trade(&mut bid).is_ok());
            assert!(!ask.is_closed());
            assert!(bid.is_closed());
        }
    }

    mod invalid_trades {
        use super::*;

        #[test]
        fn same_side() {
            let mut ask_1: LimitOrder = Order::builder()
                .side(OrderSide::Ask)
                .limit(dec!(10), dec!(10))
                .build()
                .try_into()
                .unwrap();
            let mut ask_2 = Order::builder()
                .side(OrderSide::Ask)
                .limit(dec!(10), dec!(10))
                .build();

            assert!(ask_1.trade(&mut ask_2).is_err());
        }

        #[test]
        fn incompatible_prices() {
            let mut ask: LimitOrder = Order::builder()
                .side(OrderSide::Ask)
                .limit(dec!(20), dec!(10))
                .build()
                .try_into()
                .unwrap();
            let mut bid = Order::builder()
                .side(OrderSide::Bid)
                .limit(dec!(10), dec!(10))
                .build();

            assert!(ask.trade(&mut bid).is_err());
        }
    }

    #[test]
    fn cancel_order() {
        let mut ask = Order::builder()
            .side(OrderSide::Ask)
            .limit(dec!(10), dec!(10))
            .build();
        ask.cancel();
        assert_eq!(ask.status(), OrderStatus::Cancelled);
    }

    #[test]
    fn close_order() {
        let mut ask: LimitOrder = Order::builder()
            .side(OrderSide::Ask)
            .limit(dec!(10), dec!(10))
            .build()
            .try_into()
            .unwrap();
        let mut bid = Order::builder()
            .side(OrderSide::Bid)
            .limit(dec!(10), dec!(5))
            .build();

        assert!(ask.trade(&mut bid).is_ok());

        ask.cancel();

        assert_eq!(ask.status(), OrderStatus::Closed);
    }
}
