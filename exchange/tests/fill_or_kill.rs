use exchange_algo::Orderbook;
use exchange_core::Exchange;
use exchange_types::{Order, OrderId, OrderSide, OrderType, TimeInForce};
use tap::{Pipe, Tap};

#[test]
fn valid_match() {
    let mut exchange = Orderbook::new().tap_mut(|exchange| {
        let limit_order = Order::builder()
            .side(OrderSide::Ask)
            .limit(100, 100)
            .build();

        assert!(exchange.matching(limit_order).is_ok());
    });

    let fill_or_kill = OrderId::random().pipe(|order_id| {
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

    assert!(exchange.matching(fill_or_kill).is_ok());
}

#[test]
fn invalid_match() {
    let mut exchange = Orderbook::new().tap_mut(|exchange| {
        let limit_order =
            Order::builder().side(OrderSide::Ask).limit(100, 10).build();

        assert!(exchange.matching(limit_order).is_ok());
    });

    let fill_or_kill = OrderId::random().pipe(|order_id| {
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

    assert!(exchange.matching(fill_or_kill).is_ok());
}
