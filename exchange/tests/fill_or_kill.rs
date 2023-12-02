use exchange_algo::Orderbook;
use exchange_core::Exchange;
use exchange_types::{Order, OrderSide};
use tap::Tap;

#[test]
fn valid_match() {
    let mut exchange = Orderbook::new().tap_mut(|exchange| {
        let limit_order = Order::builder()
            .side(OrderSide::Ask)
            .limit(100, 100)
            .build();

        assert!(exchange.matching(limit_order).is_ok());
    });

    let fill_or_kill = Order::builder()
        .side(OrderSide::Bid)
        .limit(100, 100)
        .ioc()
        .all_or_none()
        .build();

    assert!(exchange.matching(fill_or_kill).is_ok());
}

#[test]
fn invalid_match() {
    let mut exchange = Orderbook::new().tap_mut(|exchange| {
        let limit_order =
            Order::builder().side(OrderSide::Ask).limit(100, 10).build();

        assert!(exchange.matching(limit_order).is_ok());
    });

    let fill_or_kill = Order::builder()
        .side(OrderSide::Bid)
        .limit(100, 100)
        .ioc()
        .all_or_none()
        .build();

    assert!(exchange.matching(fill_or_kill).is_ok());
}
