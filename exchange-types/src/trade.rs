use either::Either;
use exchange_core::Asset as _;
use exchange_core::Trade as _;

use crate::error::TradeError;
use crate::LimitOrder;
use crate::Notional;
use crate::Order;
use crate::OrderId;
use crate::Price;
use crate::Quantity;

#[derive(Debug)]
#[cfg_attr(test, derive(Copy, Clone))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trade {
    pub(crate) taker: OrderId,
    pub(crate) maker: OrderId,
    /// Amount exchanged.
    pub(crate) quantity: Quantity,
    /// Traded price.
    pub(crate) price: Price,
    /// Total value of the underlying trade.
    pub(crate) notional: Notional,
}

impl Trade {
    /// Constructs a new `Trade`, returning an error if something fails.
    #[track_caller]
    pub fn try_new(
        maker: &mut LimitOrder,
        taker: &mut Order,
    ) -> Result<Trade, TradeError> {
        maker.matches(&*taker)?;

        let price =
            maker.limit_price().expect("maker must always have a price");

        let exchanged = match taker.remaining() {
            Either::Left(funds) => funds / price,
            Either::Right(quantity) => quantity,
        }
        .min(maker.remaining());

        maker.fill(exchanged);
        taker.fill(exchanged, price);

        Ok(Trade {
            taker: taker.id(),
            maker: maker.id(),
            quantity: exchanged,
            price,
            notional: exchanged * price,
        })
    }

    /// Returns the amount exchanged.
    #[inline]
    pub const fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Returns the traded price.
    #[inline]
    pub const fn price(&self) -> Price {
        self.price
    }

    /// Returns the total value of the underlying trade.
    #[inline]
    pub const fn notional(&self) -> Notional {
        self.notional
    }
}
