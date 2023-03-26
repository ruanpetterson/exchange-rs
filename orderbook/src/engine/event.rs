use orderbook_core::{Asset, ExchangeEvent};

pub enum Event<Order: Asset> {
    Added(<Order as Asset>::OrderId),
    Removed(<Order as Asset>::OrderId),
    Traded(<Order as Asset>::Trade),
}

impl<Order: Asset> ExchangeEvent for Event<Order> {
    type Order = Order;

    #[inline]
    fn added(order_id: <Self::Order as Asset>::OrderId) -> Self {
        Self::Added(order_id)
    }

    #[inline]
    fn removed(order_id: <Self::Order as Asset>::OrderId) -> Self {
        Self::Removed(order_id)
    }

    #[inline]
    fn traded(trade: <Self::Order as Asset>::Trade) -> Self {
        Self::Traded(trade)
    }
}
