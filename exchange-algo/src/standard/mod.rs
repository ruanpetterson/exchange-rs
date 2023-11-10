pub mod orderbook;
mod policy;

use exchange_core::{Algo, Asset, Exchange, ExchangeExt, Opposite};

pub struct MatchingAlgo;
impl Algo for MatchingAlgo {
    type Error = DefaultExchangeError;
    type Output = ();

    #[inline]
    fn matching<E: Exchange + ExchangeExt>(
        exchange: &mut E,
        mut incoming_order: <E as Exchange>::Order,
    ) -> Result<(), DefaultExchangeError> {
        policy::before_policies()
            .iter()
            .for_each(|policy| policy(&mut incoming_order, exchange));

        while let (false, Some(top_order)) = (
            incoming_order.is_closed(),
            exchange.peek_mut(&incoming_order.side().opposite()),
        ) {
            let Ok(_trade) = incoming_order.trade(top_order) else {
                // Since incoming order is not matching to top order anymore, we
                // can move on.
                break;
            };

            if top_order.is_closed() {
                // As long as top order is completed, it can be safely removed
                // from orderbook.
                exchange
                    .pop(&incoming_order.side().opposite())
                    .expect("top order should be `Some`");
            }
        }

        policy::late_policies()
            .iter()
            .for_each(|policy| policy(&mut incoming_order, exchange));

        // If incoming order is not full-filled and open, it must be inserted
        // into the orderbook.
        if !incoming_order.is_closed() {
            // SAFETY: This call is safe because we ensure that the
            // 'incoming_order' will enter the order book if, and only if, all
            // orders on the opposite side that match with it have already been
            // executed. This is explicit at `Order::trade(&mut incoming_trade,
            // &mut top_order)` returning `Err`.
            unsafe {
                exchange.insert(incoming_order);
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DefaultExchangeError {}
