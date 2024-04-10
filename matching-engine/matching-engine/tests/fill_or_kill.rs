//! A Fill-Or-Kill order is an order to buy or sell a stock that must be
//! executed immediately in its entirety; otherwise, the entire order will be
//! cancelled (i.e., no partial execution of the order is allowed). Ih other
//! words, FOK orders are a combination of AON and IOC orders.

use exchange_core::Exchange;
use exchange_types::Order;
use exchange_types::OrderSide;
use matching_engine_algo::Orderbook;
use rust_decimal_macros::dec;
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
                    .limit(dec!(100), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .market(dec!(100))
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange);
        }

        #[test]
        fn double_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(dec!(100), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(dec!(200), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .market(dec!(200))
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange);
        }

        #[test]
        fn triple_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(dec!(100), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(dec!(200), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(dec!(300), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            let fill_or_kill = Order::builder()
                .side(OrderSide::Ask)
                .market(dec!(300))
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange);
        }
    }

    mod same_prices {
        use super::*;

        #[test]
        fn single_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(dec!(90), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .limit(dec!(100), dec!(100))
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange);
        }

        #[test]
        fn double_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(dec!(80), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(dec!(90), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .limit(dec!(100), dec!(200))
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange);
        }

        #[test]
        fn triple_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(dec!(100), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(dec!(110), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(dec!(120), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            let fill_or_kill = Order::builder()
                .side(OrderSide::Ask)
                .limit(dec!(90), dec!(300))
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange);
        }
    }

    mod different_prices {
        use super::*;

        #[test]
        fn single_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(dec!(100), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .limit(dec!(100), dec!(100))
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange);
        }

        #[test]
        fn double_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(dec!(100), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Ask)
                    .limit(dec!(100), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            let fill_or_kill = Order::builder()
                .side(OrderSide::Bid)
                .limit(dec!(100), dec!(200))
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange);
        }

        #[test]
        fn triple_match() {
            let mut exchange = Orderbook::new().tap_mut(|exchange| {
                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(dec!(100), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(dec!(100), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());

                let limit_order = Order::builder()
                    .side(OrderSide::Bid)
                    .limit(dec!(100), dec!(100))
                    .build();

                assert!(exchange.matching(limit_order).is_ok());
            });

            let fill_or_kill = Order::builder()
                .side(OrderSide::Ask)
                .limit(dec!(100), dec!(300))
                .ioc()
                .all_or_none()
                .build();

            assert!(exchange.matching(fill_or_kill).is_ok());

            insta::assert_debug_snapshot!(&exchange);
        }
    }
}

mod invalid {
    use super::*;

    #[test]
    fn amount_mismatch() {
        let mut exchange = Orderbook::new().tap_mut(|exchange| {
            let limit_order = Order::builder()
                .side(OrderSide::Ask)
                .limit(dec!(100), dec!(50))
                .build();

            assert!(exchange.matching(limit_order).is_ok());
        });

        let fill_or_kill = Order::builder()
            .side(OrderSide::Bid)
            .limit(dec!(100), dec!(100))
            .ioc()
            .all_or_none()
            .build();

        assert!(exchange.matching(fill_or_kill).is_ok());

        insta::assert_debug_snapshot!(&exchange);
    }

    #[test]
    fn price_mismatch() {
        let mut exchange = Orderbook::new().tap_mut(|exchange| {
            let limit_order = Order::builder()
                .side(OrderSide::Bid)
                .limit(dec!(50), dec!(50))
                .build();

            assert!(exchange.matching(limit_order).is_ok());

            let limit_order = Order::builder()
                .side(OrderSide::Bid)
                .limit(dec!(100), dec!(50))
                .build();

            assert!(exchange.matching(limit_order).is_ok());
        });

        let fill_or_kill = Order::builder()
            .side(OrderSide::Ask)
            .limit(dec!(100), dec!(100))
            .ioc()
            .all_or_none()
            .build();

        assert!(exchange.matching(fill_or_kill).is_ok());

        insta::assert_debug_snapshot!(&exchange);
    }
}
