use compact_str::CompactString;
use once_cell::sync::Lazy;
use orderbook_core::{Asset, Exchange};

use crate::engine::{Event, Order, Orderbook, Trade};

const PAIR: CompactString = CompactString::new_inline("BTC/USDC");
const ORDERS: Lazy<Box<[Order]>> = Lazy::new(|| {
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
    assert_eq!(trade.price, ask.limit_price().max(bid.limit_price()));
}

#[test]
fn taker_advantage_for_bid() {
    let mut bid = ORDERS[2];
    let mut ask = ORDERS[3];

    let trade = bid.trade(&mut ask).expect("a sucessful trade");
    assert_eq!(trade.price, ask.limit_price().min(bid.limit_price()));
}

#[test]
fn orderbook() {
    let mut orderbook = Orderbook::<Order, Event<Order>, Trade>::new(&PAIR);

    assert_eq!(orderbook.matching(ORDERS[0]).len(), 1);
    assert_eq!(orderbook.matching(ORDERS[1]).len(), 1);
    assert_eq!(orderbook.matching(ORDERS[2]).len(), 1);
    assert_eq!(orderbook.matching(ORDERS[3]).len(), 2);
    assert_eq!(orderbook.matching(ORDERS[4]).len(), 1);
    assert_eq!(orderbook.matching(ORDERS[5]).len(), 2);
}
