use std::ops::ControlFlow;

use either::Either;
use exchange_core::Asset;
use exchange_core::Exchange;
use exchange_core::Opposite;
use exchange_core::Trade;
use num::Zero;

use super::seq;
use super::Policy;

pub(super) struct FillOrKill;
impl<O, E> Policy<O, E, seq::Before> for FillOrKill
where
    E: Exchange,
    <E as Exchange>::Order: Trade<O>,
    O: Asset<
        OrderId = <<E as Exchange>::Order as Asset>::OrderId,
        OrderNotional = <<E as Exchange>::Order as Asset>::OrderNotional,
        OrderPrice = <<E as Exchange>::Order as Asset>::OrderPrice,
        OrderQuantity = <<E as Exchange>::Order as Asset>::OrderQuantity,
        OrderSide = <<E as Exchange>::Order as Asset>::OrderSide,
        OrderStatus = <<E as Exchange>::Order as Asset>::OrderStatus,
    >,
{
    #[inline]
    fn enforce(&self, incoming_order: &mut O, exchange: &E) {
        if incoming_order.is_fill_or_kill()
            && !FillOrKill::can_fill(incoming_order, exchange)
        {
            // The exchange should possess a sufficient number of orders to
            // execute an all-or-none order; otherwise, the all-or-none
            // order must be cancelled.
            incoming_order.cancel();
        }
    }
}

impl FillOrKill {
    /// Returns if `incoming_order` can be completely filled within given
    /// exchange.
    ///
    /// `incoming_order` is the order you want to check if it can be filled,
    /// and `exchange` is the orderbook that we'll use to compare against
    /// the given order.
    ///
    /// `can_fill()` is short-circuiting; in other words, it will stop
    /// processing as soon as it ensures the given order can be full-filled,
    /// given that no matter what else happens, the result will also be
    /// `true`.
    #[inline]
    fn can_fill<O, E>(incoming_order: &O, exchange: &E) -> bool
    where
        E: Exchange,
        <E as Exchange>::Order: Trade<O>,
        O: Asset<
            OrderId = <<E as Exchange>::Order as Asset>::OrderId,
            OrderNotional = <<E as Exchange>::Order as Asset>::OrderNotional,
            OrderPrice = <<E as Exchange>::Order as Asset>::OrderPrice,
            OrderQuantity = <<E as Exchange>::Order as Asset>::OrderQuantity,
            OrderSide = <<E as Exchange>::Order as Asset>::OrderSide,
            OrderStatus = <<E as Exchange>::Order as Asset>::OrderStatus,
        >,
    {
        let mut iter = exchange
            .iter(&incoming_order.side().opposite())
            .take_while(|order| {
                // Gather only the orders that are compatible to the
                // `incoming_order`.
                order.matches(incoming_order).is_ok()
            })
            .map(|order| {
                let Either::Right(remaining) = order.remaining() else {
                    unreachable!();
                };

                (
                    order
                        .limit_price()
                        .expect("maker orders always have a limit price"),
                    remaining,
                )
            });

        match incoming_order.remaining() {
            Either::Left(remaining) => {
                iter.try_fold(
                    remaining,
                    |mut remaining, (limit_price, available_to_trade)| {
                        let available_to_trade =
                            limit_price * available_to_trade;

                        remaining =
                            remaining - available_to_trade.min(remaining);

                        // This means that the `incoming_order` can be fully
                        // filled.
                        if remaining.is_zero() {
                            // Using `ControlFlow` make this call
                            // short-circuiting; in other words, it will stop
                            // processing as soon as the closure returns
                            // `ControlFlow::Break`.
                            return ControlFlow::Break(remaining);
                        }

                        ControlFlow::Continue(remaining)
                    },
                )
                .is_break()
            }
            Either::Right(remaining) => {
                iter.try_fold(
                    remaining,
                    |mut remaining, (_limit_price, available_to_trade)| {
                        remaining =
                            remaining - available_to_trade.min(remaining);

                        // This means that the `incoming_order` can be fully
                        // filled.
                        if remaining.is_zero() {
                            // Using `ControlFlow` make this call
                            // short-circuiting; in other words, it will stop
                            // processing as soon as the closure returns
                            // `ControlFlow::Break`.
                            return ControlFlow::Break(remaining);
                        }

                        ControlFlow::Continue(remaining)
                    },
                )
                .is_break()
            }
        }
    }
}
