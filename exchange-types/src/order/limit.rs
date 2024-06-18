use std::borrow::Borrow;
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::ops::AddAssign as _;

use either::Either;
use exchange_core::Asset;
use exchange_core::Trade;

use crate::error::ConversionError;
use crate::error::OrderError;
use crate::error::PriceError;
use crate::error::StatusError;
use crate::error::TradeError;
use crate::order_type::ByBase;
use crate::Notional;
use crate::Order;
use crate::OrderId;
use crate::OrderSide;
use crate::OrderStatus;
use crate::OrderType;
use crate::Price;
use crate::Quantity;
use crate::TimeInForce;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LimitOrder {
    id: OrderId,
    side: OrderSide,
    unit_price: Price,
    /// The post-only flag indicates that the order should only make
    /// liquidity. If any part of the order results in taking liquidity,
    /// the order will be rejected and no part of it will execute.
    post_only: bool,
    quantity: Quantity,
    #[cfg_attr(feature = "serde", serde(default))]
    filled: Quantity,
    status: OrderStatus,
}

impl LimitOrder {
    /// Fill an order within the specified amount.
    ///
    /// # Panics
    ///
    /// Panics if `amount` is greater then `remaining`.
    #[inline]
    pub(crate) fn fill(&mut self, quantity: Quantity) {
        self.try_fill(quantity)
            .expect("order does not have available amount to fill")
    }

    /// Fill an order within the specified amount.
    ///
    /// # Safety
    ///
    /// This results in an unreliable state when current `Order::filled`
    /// overflows `Order::amount` or given amount is zero.
    #[inline]
    pub(crate) unsafe fn fill_unchecked(&mut self, quantity: Quantity) {
        self.filled.add_assign(quantity);

        self.status = if self.remaining().is_zero() {
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
    ) -> Result<(), OrderError> {
        if quantity.is_zero() {
            return Err(OrderError::NoFill);
        }

        if quantity > self.remaining() {
            return Err(OrderError::Overfill);
        }

        // SAFETY: we already guarantee that `remaining >= amount > 0`.
        unsafe { self.fill_unchecked(quantity) };

        Ok(())
    }

    #[inline]
    pub fn remaining(&self) -> Quantity {
        self.quantity - self.filled
    }
}

impl Borrow<LimitOrder> for Reverse<LimitOrder> {
    #[inline]
    fn borrow(&self) -> &LimitOrder {
        &self.0
    }
}

impl PartialEq for LimitOrder {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}
impl Eq for LimitOrder {}

impl PartialOrd for LimitOrder {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.limit_price().partial_cmp(&other.limit_price())
    }
}

impl Asset for LimitOrder {
    type OrderId = OrderId;
    type OrderNotional = Notional;
    type OrderPrice = Price;
    type OrderQuantity = Quantity;
    type OrderSide = OrderSide;
    type OrderStatus = OrderStatus;
    type Trade = crate::Trade;
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
        Some(self.unit_price)
    }

    #[inline]
    fn remaining(&self) -> Either<Self::OrderNotional, Self::OrderQuantity> {
        let remaining = self.remaining();
        Either::Right(remaining)
    }

    #[inline]
    fn status(&self) -> OrderStatus {
        self.status
    }

    #[inline]
    fn is_fill_or_kill(&self) -> bool {
        false
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
        false
    }

    #[inline]
    fn is_post_only(&self) -> bool {
        self.post_only
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

impl Trade<Order> for LimitOrder {
    #[inline]
    fn trade(
        &mut self,
        other: &mut Order,
    ) -> Result<Self::Trade, Self::TradeError> {
        let (maker, taker) = (self, other);

        Self::Trade::try_new(maker, taker)
    }

    #[inline]
    fn matches(&self, other: &Order) -> Result<(), Self::TradeError> {
        let (maker, taker) = (self, other);

        // Matching cannot occur between closed orders.
        if taker.is_closed() || maker.is_closed() {
            return Err(StatusError::Closed)?;
        }

        let maker_limit_price = maker
            .limit_price()
            .expect("market makers always have a limit price");

        let Some(taker_limit_price) = taker.limit_price() else {
            return Ok(());
        };

        let (ask_price, bid_price) = match (taker.side(), maker.side()) {
            (OrderSide::Ask, OrderSide::Bid) => {
                (taker_limit_price, maker_limit_price)
            }
            (OrderSide::Bid, OrderSide::Ask) => {
                (maker_limit_price, taker_limit_price)
            }
            _ => return Err(TradeError::SameSide)?,
        };

        (bid_price >= ask_price)
            .then_some(())
            .ok_or(PriceError::Incompatible)
            .map_err(Into::into)
    }
}

impl From<LimitOrder> for Order {
    #[inline]
    fn from(order: LimitOrder) -> Order {
        Order {
            id: order.id,
            side: order.side,
            type_: OrderType::Limit {
                limit_price: order.unit_price,
                time_in_force: TimeInForce::GoodTillCancel {
                    post_only: order.post_only,
                },
                priced_by: ByBase {
                    quantity: order.quantity,
                    filled: order.filled,
                },
            },
            status: order.status,
        }
    }
}

impl TryFrom<Order> for LimitOrder {
    type Error = ConversionError;

    fn try_from(order: Order) -> Result<Self, Self::Error> {
        let OrderType::Limit {
            limit_price,
            time_in_force: TimeInForce::GoodTillCancel { post_only },
            priced_by,
        } = order.type_
        else {
            return Err(ConversionError::Incompatible)?;
        };

        Ok(LimitOrder {
            id: order.id,
            side: order.side,
            unit_price: limit_price,
            post_only,
            quantity: priced_by.quantity,
            filled: priced_by.filled,
            status: order.status,
        })
    }
}
