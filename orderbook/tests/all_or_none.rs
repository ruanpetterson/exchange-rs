use orderbook_algo::DefaultExchange;
use orderbook_core::OrderSide;
use orderbook_types::{Order, OrderId, Orderbook, TimeInForce};
use tap::{Pipe, Tap};

#[test]
fn valid_match() {
    let mut exchange =
        Orderbook::new()
            .pipe(DefaultExchange::from)
            .tap_mut(|exchange| {
                let order_id = OrderId::new(1);
                let limit_order =
                    Order::new_limit(order_id, OrderSide::Ask, 100, 100);

                assert!(exchange.matching(limit_order).is_ok());
            });

    let all_or_none = OrderId::new(1).pipe(|order_id| {
        Order::new(
            order_id,
            OrderSide::Bid,
            orderbook_types::OrderType::Limit {
                limit_price: 100,
                time_in_force: TimeInForce::ImmediateOrCancel {
                    all_or_none: true,
                },
            },
            100,
        )
    });

    assert!(exchange.matching(all_or_none).is_ok());
}

#[test]
fn invalid_match() {
    let mut exchange =
        Orderbook::new()
            .pipe(DefaultExchange::from)
            .tap_mut(|exchange| {
                let order_id = OrderId::new(1);
                let limit_order =
                    Order::new_limit(order_id, OrderSide::Ask, 100, 10);

                assert!(exchange.matching(limit_order).is_ok());
            });

    let all_or_none = OrderId::new(1).pipe(|order_id| {
        Order::new(
            order_id,
            OrderSide::Bid,
            orderbook_types::OrderType::Limit {
                limit_price: 100,
                time_in_force: TimeInForce::ImmediateOrCancel {
                    all_or_none: true,
                },
            },
            100,
        )
    });

    assert!(exchange.matching(all_or_none).is_ok());
}
