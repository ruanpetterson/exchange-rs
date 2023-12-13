use compact_str::CompactString;
use exchange_algo::Orderbook;
use exchange_core::Exchange;
use exchange_types::Order;
use thiserror::Error;

mod request;
pub use request::Request;

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
        incoming_order: Request<Order>,
    ) -> Result<(), EngineError> {
        match incoming_order {
            Request::Create { ref pair, order } => {
                if pair != &self.pair {
                    Err(PairError::Mismatch {
                        expected: self.pair.clone(),
                        found: pair.clone(),
                    })?;
                }

                let _ = self.orderbook.matching(order);
            }
            Request::Delete { order_id } => {
                self.orderbook.remove(&order_id);
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
