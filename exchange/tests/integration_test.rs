use exchange_core::Asset;
use exchange_types::{Order, Orderbook, Trade};
use once_cell::sync::Lazy;

static ORDERS: Lazy<Box<[Order]>> = Lazy::new(|| {
    let input = include_str!("./mock_orders.json");
    serde_json::from_str(input).expect("a set of valid orders")
});

#[test]
fn simple_match() {
    let mut ask = ORDERS[0];
    let mut bid = ORDERS[1];

    assert!(ask.trade(&mut bid).is_some());
    assert!(ask.is_closed());
    assert!(bid.is_closed());
}

#[test]
fn partial_match() {
    let mut ask = ORDERS[3];
    let mut bid = ORDERS[2];

    assert!(ask.trade(&mut bid).is_some());
    assert!(!ask.is_closed());
    assert!(bid.is_closed());
}

#[test]
fn taker_advantage_for_ask() {
    let mut ask = ORDERS[3];
    let mut bid = ORDERS[2];

    let trade = ask.trade(&mut bid).expect("a sucessful trade");
    assert_eq!(
        trade.price(),
        ask.limit_price().unwrap().max(bid.limit_price().unwrap())
    );
}

#[test]
fn taker_advantage_for_bid() {
    let mut bid = ORDERS[2];
    let mut ask = ORDERS[3];

    let trade = bid.trade(&mut ask).expect("a sucessful trade");
    assert_eq!(
        trade.price(),
        ask.limit_price().unwrap().min(bid.limit_price().unwrap())
    );
}

#[test]
fn orderbook() {
    let mut _orderbook = Orderbook::<Order, Trade>::new();
}
