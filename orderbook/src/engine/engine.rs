use orderbook_core::{Asset, Exchange};

use super::{Event, Order, OrderId, OrderRequest, Orderbook, Trade};

pub struct Engine {
    orderbook: Orderbook<Order, Event<Order>, Trade>,
}

impl Engine {
    #[inline]
    pub fn new(pair: &str) -> Self {
        Self {
            orderbook: Orderbook::new(pair),
        }
    }

    #[inline]
    pub fn process(
        &mut self,
        incoming_order: OrderRequest,
    ) -> Vec<<Orderbook<Order, Event<Order>, Trade> as Exchange>::Event> {
        match incoming_order {
            OrderRequest::Create { .. } => {
                let order = Order::try_from(incoming_order).unwrap();
                self.orderbook.matching(order)
            }
            OrderRequest::Delete { ref order_id } => {
                if let Some(order) = self
                    .orderbook
                    .remove(&OrderId::new(order_id.parse::<u64>().unwrap()))
                {
                    vec![Event::Removed(order.id())]
                } else {
                    vec![]
                }
            }
        }
    }

    #[inline]
    pub fn orderbook(&self) -> &Orderbook<Order, Event<Order>, Trade> {
        &self.orderbook
    }
}
