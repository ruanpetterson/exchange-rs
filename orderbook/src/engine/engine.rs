use orderbook_algo::DefaultExchange;
use orderbook_core::Exchange;

use super::{Order, OrderId, OrderRequest, Orderbook, Trade};

pub struct Engine {
    orderbook: DefaultExchange<Orderbook<Order, Trade>>,
}

impl Engine {
    #[inline]
    pub fn new(pair: &str) -> Self {
        Self {
            orderbook: Orderbook::new(pair).into(),
        }
    }

    #[inline]
    pub fn process(&mut self, incoming_order: OrderRequest) {
        match incoming_order {
            OrderRequest::Create { .. } => {
                let order = Order::try_from(incoming_order).unwrap();
                let _ = self.orderbook.matching(order);
            }
            OrderRequest::Delete { ref order_id } => {
                self.orderbook
                    .remove(&OrderId::new(order_id.parse::<u64>().unwrap()));
            }
        }
    }

    #[inline]
    pub fn orderbook(&self) -> &Orderbook<Order, Trade> {
        &self.orderbook
    }
}
