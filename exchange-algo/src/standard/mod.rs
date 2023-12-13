pub mod orderbook;
mod policy;

use exchange_core::{Algo, Asset, Exchange, ExchangeExt, Opposite};

pub struct MatchingAlgo;
impl Algo for MatchingAlgo {
    type Error = DefaultExchangeError;
    type Output = ();

    #[inline]
    fn matching<E>(
        exchange: &mut E,
        mut incoming_order: <E as Exchange>::Order,
    ) -> Result<(), DefaultExchangeError>
    where
        E: Exchange + ExchangeExt,
    {
        policy::before_policies()
            .iter()
            .for_each(|policy| policy(&mut incoming_order, exchange));

        while !incoming_order.is_closed() {
            let Some(mut top_order) =
                exchange.peek_mut(&incoming_order.side().opposite())
            else {
                // Since there is no opposite order anymore, we can move on.
                break;
            };

            let Ok(_trade) = incoming_order.trade(&mut *top_order) else {
                // Since incoming order is not matching to top order
                // anymore, we can also move on.
                break;
            };

            if top_order.is_closed() {
                let top_order_id = top_order.id();

                // We must explicity drop to reuse the `exchange`.
                drop(top_order);

                // As long as top order is completed, it can be safely removed
                // from orderbook.
                exchange
                    .remove(&top_order_id)
                    .expect("order should be `Some`");
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
