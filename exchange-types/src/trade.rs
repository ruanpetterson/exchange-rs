use exchange_core::Asset as _;
use exchange_core::Trade as _;

use crate::error::TradeError;
use crate::Amount;
use crate::LimitOrder;
use crate::Notional;
use crate::Order;
use crate::OrderId;
use crate::Price;

#[derive(Debug)]
#[cfg_attr(test, derive(Copy, Clone))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trade {
    pub(crate) taker: OrderId,
    pub(crate) maker: OrderId,
    /// Amount exchanged.
    pub(crate) amount: Amount,
    /// Traded price.
    pub(crate) price: Price,
    /// Total value of the underlying trade.
    pub(crate) notional: Notional,
}

impl Trade {
    /// Constructs a new `Trade`, returning an error if something fails.
    #[inline]
    pub fn try_new(
        maker: &mut LimitOrder,
        taker: &mut Order,
    ) -> Result<Trade, TradeError> {
        maker.matches(&*taker)?;

        let exchanged = taker.remaining().min(maker.remaining());
        let price =
            maker.limit_price().expect("maker must always have a price");

        maker.fill(exchanged);
        taker.fill(exchanged);

        Ok(Trade {
            taker: taker.id(),
            maker: maker.id(),
            amount: exchanged,
            price,
            notional: exchanged * price,
        })
    }

    /// Returns the amount exchanged.
    #[inline]
    pub const fn amount(&self) -> Amount {
        self.amount
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
