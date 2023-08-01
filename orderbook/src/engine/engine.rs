use compact_str::CompactString;
use orderbook_algo::DefaultExchange;
use orderbook_core::Exchange;
use thiserror::Error;

use super::{Order, OrderId, OrderRequest, Orderbook, Trade};

pub struct Engine {
    pair: CompactString,
    orderbook: DefaultExchange<Orderbook<Order, Trade>>,
}

impl Engine {
    #[inline]
    pub fn new(pair: &str) -> Self {
        Self {
            pair: CompactString::new_inline(pair),
            orderbook: Orderbook::new().into(),
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
            OrderRequest::Delete { ref order_id } => {
                self.orderbook
                    .remove(&OrderId::new(order_id.parse::<u64>().unwrap()));
            }
        };

        Ok(())
    }

    #[inline]
    pub fn orderbook(&self) -> &Orderbook<Order, Trade> {
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
