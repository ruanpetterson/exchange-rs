use exchange_core::Exchange;
use exchange_types::{
    Order, OrderId, OrderSide, OrderType, Orderbook, TimeInForce,
};
use tap::{Pipe, Tap};

#[test]
fn valid_match() {
    let mut exchange = Orderbook::new().tap_mut(|exchange| {
        let order_id = OrderId::new(1);
        let limit_order = Order::new_limit(order_id, OrderSide::Ask, 100, 100);

        assert!(exchange.matching(limit_order).is_ok());
    });

    let all_or_none = OrderId::new(1).pipe(|order_id| {
        Order::new(
            order_id,
            OrderSide::Bid,
            OrderType::Limit {
                limit_price: 100,
                time_in_force: TimeInForce::ImmediateOrCancel {
                    all_or_none: true,
                },
                amount: 100,
                filled: 0,
            },
        )
    });

    assert!(exchange.matching(all_or_none).is_ok());
}

#[test]
fn invalid_match() {
    let mut exchange = Orderbook::new().tap_mut(|exchange| {
        let order_id = OrderId::new(1);
        let limit_order = Order::new_limit(order_id, OrderSide::Ask, 100, 10);

        assert!(exchange.matching(limit_order).is_ok());
    });

    let all_or_none = OrderId::new(1).pipe(|order_id| {
        Order::new(
            order_id,
            OrderSide::Bid,
            OrderType::Limit {
                limit_price: 100,
                time_in_force: TimeInForce::ImmediateOrCancel {
                    all_or_none: true,
                },
                amount: 100,
                filled: 0,
            },
        )
    });

    assert!(exchange.matching(all_or_none).is_ok());
}
