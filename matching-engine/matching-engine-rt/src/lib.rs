use compact_str::CompactString;
use exchange_core::Exchange;
use exchange_types::Order;
use exchange_types::OrderId;
use exchange_types::OrderRequest;
use matching_engine_algo::Orderbook;
use thiserror::Error;

pub struct Engine {
    symbol: CompactString,
    orderbook: Orderbook,
}

impl Engine {
    #[inline]
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: CompactString::new_inline(symbol),
            orderbook: Orderbook::new(),
        }
    }

    pub fn process(
        &mut self,
        incoming_order: OrderRequest,
    ) -> Result<(), EngineError> {
        match incoming_order {
            OrderRequest::Create { ref symbol, .. } => {
                if symbol != &self.symbol {
                    Err(SymbolError::Mismatch {
                        expected: self.symbol.clone(),
                        found: symbol.clone(),
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
    SymbolError(#[from] SymbolError),
}

#[derive(Debug, Error)]
pub enum SymbolError {
    #[error("symbol mismatch (expected={}, found={})", .expected, .found)]
    Mismatch {
        expected: CompactString,
        found: CompactString,
    },
}
