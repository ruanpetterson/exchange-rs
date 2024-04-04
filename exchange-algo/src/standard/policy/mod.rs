mod fill_or_kill;
mod immediate_or_cancel;
mod post_only;
mod seq {
    pub(super) trait Seq {}

    pub(super) enum Before {}
    impl Seq for Before {}

    pub(super) enum Late {}
    impl Seq for Late {}
}

use exchange_core::{Asset, Exchange, ExchangeExt, Trade};

use self::fill_or_kill::FillOrKill;
use self::immediate_or_cancel::ImmediateOrCancel;
use self::post_only::PostOnly;

#[allow(private_bounds)]
pub(crate) trait Policy<O, E, S>
where
    E: Exchange,
    <E as Exchange>::Order: Trade<O>,
    S: seq::Seq,
    O: Asset<
        OrderAmount = <<E as Exchange>::Order as Asset>::OrderAmount,
        OrderId = <<E as Exchange>::Order as Asset>::OrderId,
        OrderPrice = <<E as Exchange>::Order as Asset>::OrderPrice,
        OrderSide = <<E as Exchange>::Order as Asset>::OrderSide,
        OrderStatus = <<E as Exchange>::Order as Asset>::OrderStatus,
    >,
{
    fn enforce(order: &mut O, exchange: &E);
}

/// Policies that should be run before matching.
#[inline]
pub(super) const fn before_policies<'e, O, E>() -> &'e [fn(&mut O, &E)]
where
    E: Exchange + ExchangeExt + 'e,
    <E as Exchange>::Order: Trade<O>,
    O: Asset<
        OrderAmount = <<E as Exchange>::Order as Asset>::OrderAmount,
        OrderId = <<E as Exchange>::Order as Asset>::OrderId,
        OrderPrice = <<E as Exchange>::Order as Asset>::OrderPrice,
        OrderSide = <<E as Exchange>::Order as Asset>::OrderSide,
        OrderStatus = <<E as Exchange>::Order as Asset>::OrderStatus,
    >,
{
    &[
        <FillOrKill as Policy<O, E, _>>::enforce,
        <PostOnly as Policy<O, E, _>>::enforce,
    ]
}

/// Policies that should be run after matching.
#[inline]
pub(super) const fn late_policies<'e, O, E>() -> &'e [fn(&mut O, &E)]
where
    E: Exchange + ExchangeExt + 'e,
    <E as Exchange>::Order: Trade<O>,
    O: Asset<
        OrderAmount = <<E as Exchange>::Order as Asset>::OrderAmount,
        OrderId = <<E as Exchange>::Order as Asset>::OrderId,
        OrderPrice = <<E as Exchange>::Order as Asset>::OrderPrice,
        OrderSide = <<E as Exchange>::Order as Asset>::OrderSide,
        OrderStatus = <<E as Exchange>::Order as Asset>::OrderStatus,
    >,
{
    &[<ImmediateOrCancel as Policy<O, E, _>>::enforce]
}
