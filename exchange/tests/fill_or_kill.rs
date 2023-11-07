use exchange_core::Exchange;
use exchange_types::{Id, Kind, Order, Orderbook, Side, TimeInForce};
use tap::{Pipe, Tap};

#[test]
fn valid_match() {
    let mut exchange = Orderbook::new().tap_mut(|exchange| {
        let order_id = Id::new(1);
        let limit_order = Order::new_limit(order_id, Side::Ask, 100, 100);

        assert!(exchange.matching(limit_order).is_ok());
    });

    let fill_or_kill = Id::new(1).pipe(|order_id| {
        Order::new(
            order_id,
            Side::Bid,
            Kind::Limit {
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
        let order_id = Id::new(1);
        let limit_order = Order::new_limit(order_id, Side::Ask, 100, 10);

        assert!(exchange.matching(limit_order).is_ok());
    });

    let fill_or_kill = Id::new(1).pipe(|order_id| {
        Order::new(
            order_id,
            Side::Bid,
            Kind::Limit {
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
