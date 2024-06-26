//! A Fill-Or-Kill order is an order to buy or sell a stock that must be
//! executed immediately in its entirety; otherwise, the entire order will be
//! cancelled (i.e., no partial execution of the order is allowed). Ih other
//! words, FOK orders are a combination of AON and IOC orders.

use exchange_core::Exchange;
use exchange_types::Order;
use exchange_types::OrderSide;
use matching_engine_algo::Orderbook;
use tap::Tap;

mod valid {
    use super::*;

    mod no_price {
        use super::*;

        #[test]
        fn single_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(100, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [
                    Order {
                        limit_price: 100,
                        remaining: 100,
                        status: Open,
                    },
                ],
                Bid: [],
            }
            "###);

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .market(100)
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [],
            }
            "###);
        }

        #[test]
        fn double_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(100, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(200, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [
                    Order {
                        limit_price: 100,
                        remaining: 100,
                        status: Open,
                    },
                    Order {
                        limit_price: 200,
                        remaining: 100,
                        status: Open,
                    },
                ],
                Bid: [],
            }
            "###);

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .market(200)
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [],
            }
            "###);
        }

        #[test]
        fn triple_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(100, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(200, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(300, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [
                    Order {
                        limit_price: 300,
                        remaining: 100,
                        status: Open,
                    },
                    Order {
                        limit_price: 200,
                        remaining: 100,
                        status: Open,
                    },
                    Order {
                        limit_price: 100,
                        remaining: 100,
                        status: Open,
                    },
                ],
            }
            "###);

            let fill_or_kill = Order::builder()
                .side(OrderSide::Ask)
                .market(300)
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [],
            }
            "###);
        }
    }

    mod same_prices {
        use super::*;

        #[test]
        fn single_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(90, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [
                    Order {
                        limit_price: 90,
                        remaining: 100,
                        status: Open,
                    },
                ],
                Bid: [],
            }
            "###);

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .limit(100, 100)
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [],
            }
            "###);
        }

        #[test]
        fn double_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(80, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(90, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [
                    Order {
                        limit_price: 80,
                        remaining: 100,
                        status: Open,
                    },
                    Order {
                        limit_price: 90,
                        remaining: 100,
                        status: Open,
                    },
                ],
                Bid: [],
            }
            "###);

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .limit(100, 200)
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [],
            }
            "###);
        }

        #[test]
        fn triple_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(100, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(110, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(120, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [
                    Order {
                        limit_price: 120,
                        remaining: 100,
                        status: Open,
                    },
                    Order {
                        limit_price: 110,
                        remaining: 100,
                        status: Open,
                    },
                    Order {
                        limit_price: 100,
                        remaining: 100,
                        status: Open,
                    },
                ],
            }
            "###);

            let fill_or_kill = Order::builder()
                .side(OrderSide::Ask)
                .limit(90, 300)
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [],
            }
            "###);
        }
    }

    mod different_prices {
        use super::*;

        #[test]
        fn single_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(100, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [
                    Order {
                        limit_price: 100,
                        remaining: 100,
                        status: Open,
                    },
                ],
                Bid: [],
            }
            "###);

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .limit(100, 100)
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [],
            }
            "###);
        }

        #[test]
        fn double_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(100, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(100, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [
                    Order {
                        limit_price: 100,
                        remaining: 100,
                        status: Open,
                    },
                    Order {
                        limit_price: 100,
                        remaining: 100,
                        status: Open,
                    },
                ],
                Bid: [],
            }
            "###);

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .limit(100, 200)
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [],
            }
            "###);
        }

        #[test]
        fn triple_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(100, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(100, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(100, 100)
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [
                    Order {
                        limit_price: 100,
                        remaining: 100,
                        status: Open,
                    },
                    Order {
                        limit_price: 100,
                        remaining: 100,
                        status: Open,
                    },
                    Order {
                        limit_price: 100,
                        remaining: 100,
                        status: Open,
                    },
                ],
            }
            "###);

            let fill_or_kill = Order::builder()
                .side(OrderSide::Ask)
                .limit(100, 300)
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange, @r###"
            {
                Ask: [],
                Bid: [],
            }
            "###);
        }
    }
}

mod invalid {
    use super::*;

    #[test]
    fn amount_mismatch() {
        let mut exchange = Orderbook::new().tap_mut(|exchange| {
            let limit_order =
                Order::builder().side(OrderSide::Ask).limit(100, 50).build();

            assert!(exchange.matching(limit_order).is_ok());
        });

        insta::assert_debug_snapshot!(&exchange, @r###"
        {
            Ask: [
                Order {
                    limit_price: 100,
                    remaining: 50,
                    status: Open,
                },
            ],
            Bid: [],
        }
        "###);

        let fill_or_kill = Order::builder()
            .side(OrderSide::Bid)
            .limit(100, 100)
            .ioc()
            .all_or_none()
            .build();

        assert!(exchange.matching(fill_or_kill).is_ok());

        insta::assert_debug_snapshot!(&exchange, @r###"
        {
            Ask: [
                Order {
                    limit_price: 100,
                    remaining: 50,
                    status: Open,
                },
            ],
            Bid: [],
        }
        "###);
    }

    #[test]
    fn price_mismatch() {
        let mut exchange = Orderbook::new().tap_mut(|exchange| {
            let limit_order =
                Order::builder().side(OrderSide::Bid).limit(50, 50).build();

            assert!(exchange.matching(limit_order).is_ok());

            let limit_order =
                Order::builder().side(OrderSide::Bid).limit(100, 50).build();

            assert!(exchange.matching(limit_order).is_ok());
        });

        insta::assert_debug_snapshot!(&exchange, @r###"
        {
            Ask: [],
            Bid: [
                Order {
                    limit_price: 100,
                    remaining: 50,
                    status: Open,
                },
                Order {
                    limit_price: 50,
                    remaining: 50,
                    status: Open,
                },
            ],
        }
        "###);

        let fill_or_kill = Order::builder()
            .side(OrderSide::Ask)
            .limit(100, 100)
            .ioc()
            .all_or_none()
            .build();

        assert!(exchange.matching(fill_or_kill).is_ok());

        insta::assert_debug_snapshot!(&exchange, @r###"
        {
            Ask: [],
            Bid: [
                Order {
                    limit_price: 100,
                    remaining: 50,
                    status: Open,
                },
                Order {
                    limit_price: 50,
                    remaining: 50,
                    status: Open,
                },
            ],
        }
        "###);
    }
}
