use compact_str::CompactString;
use exchange_core::Exchange;
use exchange_types::Order;
use exchange_types::OrderId;
use exchange_types::OrderRequest;
use matching_engine_algo::Orderbook;
use thiserror::Error;

pub struct Engine {
    pair: CompactString,
    orderbook: Orderbook,
}

impl Engine {
    #[inline]
    pub fn new(pair: &str) -> Self {
        Self {
            pair: CompactString::new_inline(pair),
            orderbook: Orderbook::new(),
        }
    }

    #[inline]
    pub fn process(
        &mut self,
        incoming_order: OrderRequest,
    ) -> Result<(), EngineError> {
        match incoming_order {
            OrderRequest::Create { ref pair, .. } => {
                if pair != &self.pair {
                    Err(PairError::Mismatch {
                        expected: self.pair.clone(),
                        found: pair.clone(),
                    })?;
                }

                let order = Order::try_from(incoming_order).unwrap();
                let _ = self.orderbook.matching(order);
            }
            OrderRequest::Delete { order_id } => {
                self.orderbook.remove(&OrderId::new(order_id));
            }
        };

        Ok(())
    }

    #[inline]
    pub fn orderbook(&self) -> &Orderbook {
        &self.orderbook
    }
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    PairError(#[from] PairError),
}

#[derive(Debug, Error)]
pub enum PairError {
    #[error("pair mismatch (expected={}, found={})", .expected, .found)]
    Mismatch {
        expected: CompactString,
        found: CompactString,
    },
}
