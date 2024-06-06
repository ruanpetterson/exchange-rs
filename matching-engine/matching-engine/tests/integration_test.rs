use assert2::assert;
use assert2::let_assert;
use exchange_core::Asset;
use exchange_core::Trade as _;
use exchange_types::LimitOrder;
use exchange_types::Order;
use matching_engine_algo::Orderbook;
use once_cell::sync::Lazy;

static ORDERS: Lazy<Box<[Order]>> = Lazy::new(|| {
    let input = include_str!("./mock_orders.json");
    serde_json::from_str(input).expect("a set of valid orders")
});

#[test]
fn simple_match() {
    let mut ask: LimitOrder = ORDERS[0].try_into().unwrap();
    let mut bid = ORDERS[1];

    let_assert!(Ok(trade) = ask.trade(&mut bid));
    assert!(ask.is_closed());
    assert!(bid.is_closed());

    insta::assert_ron_snapshot! {
        &trade,
        {
            ".taker" => "[uuid]",
            ".maker" => "[uuid]",
        },
        @r###"
    Trade(
      taker: "[uuid]",
      maker: "[uuid]",
      quantity: "100",
      price: "50000",
      notional: "5000000",
    )
    "###
    }
}

#[test]
fn partial_match() {
    let mut ask: LimitOrder = ORDERS[3].try_into().unwrap();
    let mut bid = ORDERS[2];

    let_assert!(Ok(trade) = ask.trade(&mut bid));
    assert!(!ask.is_closed());
    assert!(bid.is_closed());

    insta::assert_ron_snapshot! {
        &trade,
        {
            ".taker" => "[uuid]",
            ".maker" => "[uuid]",
        },
        @r###"
    Trade(
      taker: "[uuid]",
      maker: "[uuid]",
      quantity: "100",
      price: "50000",
      notional: "5000000",
    )
    "###
    }
}

#[test]
fn taker_advantage_for_ask() {
    let mut ask: LimitOrder = ORDERS[3].try_into().unwrap();
    let mut bid = ORDERS[2];

    let_assert!(Ok(trade) = ask.trade(&mut bid));
    assert_eq!(trade.price(), ask.limit_price().unwrap());

    insta::assert_ron_snapshot! {
        &trade,
        {
            ".taker" => "[uuid]",
            ".maker" => "[uuid]",
        },
        @r###"
    Trade(
      taker: "[uuid]",
      maker: "[uuid]",
      quantity: "100",
      price: "50000",
      notional: "5000000",
    )
    "###
    }
}

#[test]
fn taker_advantage_for_bid() {
    let mut bid: LimitOrder = ORDERS[2].try_into().unwrap();
    let mut ask = ORDERS[3];

    let_assert!(Ok(trade) = bid.trade(&mut ask));
    assert_eq!(trade.price(), bid.limit_price().unwrap());

    insta::assert_ron_snapshot! {
        &trade,
        {
            ".taker" => "[uuid]",
            ".maker" => "[uuid]",
        },
        @r###"
    Trade(
      taker: "[uuid]",
      maker: "[uuid]",
      quantity: "100",
      price: "60000",
      notional: "6000000",
    )
    "###
    }
}

#[test]
fn orderbook() {
    let mut _orderbook = Orderbook::new();
}
