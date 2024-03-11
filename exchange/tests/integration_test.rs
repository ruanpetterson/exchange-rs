use exchange_algo::Orderbook;
use exchange_core::{Asset, Trade as _};
use exchange_types::{LimitOrder, Order};
use once_cell::sync::Lazy;

static ORDERS: Lazy<Box<[Order]>> = Lazy::new(|| {
    let input = include_str!("./mock_orders.json");
    serde_json::from_str(input).expect("a set of valid orders")
});

#[test]
fn simple_match() {
    let mut ask: LimitOrder = ORDERS[0].try_into().unwrap();
    let mut bid = ORDERS[1];

    assert!(ask.trade(&mut bid).is_ok());
    assert!(ask.is_closed());
    assert!(bid.is_closed());
}

#[test]
fn partial_match() {
    let mut ask: LimitOrder = ORDERS[3].try_into().unwrap();
    let mut bid = ORDERS[2];

    assert!(ask.trade(&mut bid).is_ok());
    assert!(!ask.is_closed());
    assert!(bid.is_closed());
}

#[test]
fn taker_advantage_for_ask() {
    let mut ask: LimitOrder = ORDERS[3].try_into().unwrap();
    let mut bid = ORDERS[2];

    let trade = ask.trade(&mut bid).expect("a sucessful trade");
    assert_eq!(trade.price(), ask.limit_price().unwrap());
}

#[test]
fn taker_advantage_for_bid() {
    let mut bid: LimitOrder = ORDERS[2].try_into().unwrap();
    let mut ask = ORDERS[3];

    let trade = bid.trade(&mut ask).expect("a sucessful trade");
    assert_eq!(trade.price(), bid.limit_price().unwrap());
}

#[test]
fn orderbook() {
    let mut _orderbook = Orderbook::new();
}
