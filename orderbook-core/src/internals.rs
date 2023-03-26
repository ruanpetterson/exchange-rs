pub trait Exchange {
    type Order: Asset;
    type Event: ExchangeEvent<Order = Self::Order>;

    fn insert(&mut self, order: Self::Order);
    fn remove(
        &mut self,
        order: &<Self::Order as Asset>::OrderId,
    ) -> Option<Self::Order>;
    fn matching(&mut self, order: Self::Order) -> Vec<Self::Event> {
        let mut events = Vec::with_capacity(32);
        let mut incoming_order = order;
        while let (false, Some(top_order)) = (
            incoming_order.is_closed(),
            self.peek_mut(&incoming_order.side().opposite()),
        ) {
            debug_assert!(
                !top_order.is_closed(),
                "top order cannot be closed before try to match"
            );

            if let Some(trade) = incoming_order.trade(top_order) {
                events.push(Self::Event::traded(trade));
                match (incoming_order.is_closed(), top_order.is_closed()) {
                    (_, true) => {
                        // As long as top_order is completed, we can safely
                        // remove it from orderbook.
                        self.pop(&incoming_order.side().opposite()).expect(
                            "Remove top order because it is completed already.",
                        );
                    }
                    (true, false) => break,
                    (false, false) => unreachable!(),
                }
            } else {
                // Since incoming order is not matching to top order anymore, we
                // can move on.
                break;
            }
        }

        // We need to check if incoming order is fullfilled. If not, we'll
        // insert it into orderbook.
        if !incoming_order.is_closed() {
            events.push(Self::Event::added(incoming_order.id()));
            self.insert(incoming_order);
        }

        events
    }
    fn peek(
        &self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> Option<&Self::Order>;
    fn peek_mut(
        &mut self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> Option<&mut Self::Order>;
    fn pop(
        &mut self,
        side: &<Self::Order as Asset>::OrderSide,
    ) -> Option<Self::Order>;
}

pub trait ExchangeExt: Exchange {
    fn spread(&self) -> Option<(u64, u64)>;
    fn len(&self) -> (usize, usize);
    fn is_empty(&self) -> bool {
        self.len() == (0, 0)
    }
}

pub trait ExchangeEvent {
    type Order: Asset;
    fn added(order_id: <Self::Order as Asset>::OrderId) -> Self;
    fn removed(order_id: <Self::Order as Asset>::OrderId) -> Self;
    fn traded(trade: <Self::Order as Asset>::Trade) -> Self;
}

pub trait Asset<Order = Self>: Ord + Eq {
    /// Order unique identifier.
    type OrderId: Copy + Clone + Eq;
    /// Order current status.
    type OrderStatus: Eq + Copy + Clone;
    /// Order side.
    type OrderSide: Opposite;
    /// Trade struct.
    type Trade;
    /// Return order unique identifier.
    fn id(&self) -> Self::OrderId;
    /// Return order side.
    fn side(&self) -> Self::OrderSide;
    /// Return order limit price.
    fn limit_price(&self) -> u64;
    /// Return order remaining amount.
    fn remaining(&self) -> u64;
    /// Return current order status.
    fn status(&self) -> Self::OrderStatus;
    fn is_closed(&self) -> bool;
    fn trade(&mut self, order: &mut Order) -> Option<Self::Trade>;
    fn cancel(&mut self);
}

pub trait Opposite<Opposite = Self> {
    fn opposite(&self) -> Opposite;
}
