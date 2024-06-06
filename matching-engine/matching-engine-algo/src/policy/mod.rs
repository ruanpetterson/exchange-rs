mod fill_or_kill;
mod immediate_or_cancel;
mod post_only;
mod seq {
    pub(in crate::policy) trait Seq {}

    pub(crate) enum Before {}
    impl Seq for Before {}

    pub(crate) enum Late {}
    impl Seq for Late {}
}

use exchange_core::Asset;
use exchange_core::Exchange;
use exchange_core::ExchangeExt;
use exchange_core::Trade;

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
        OrderId = <<E as Exchange>::Order as Asset>::OrderId,
        OrderNotional = <<E as Exchange>::Order as Asset>::OrderNotional,
        OrderPrice = <<E as Exchange>::Order as Asset>::OrderPrice,
        OrderQuantity = <<E as Exchange>::Order as Asset>::OrderQuantity,
        OrderSide = <<E as Exchange>::Order as Asset>::OrderSide,
        OrderStatus = <<E as Exchange>::Order as Asset>::OrderStatus,
    >,
{
    fn enforce(&self, order: &mut O, exchange: &E);
}

/// Policies that should be run before matching.
#[inline]
pub(super) fn before_policies<'e, O, E>(
) -> &'e [&'e dyn Policy<O, E, seq::Before>]
where
    E: Exchange + ExchangeExt + 'e,
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
    const FILL_OR_KILL: &FillOrKill = &FillOrKill;
    const POST_ONLY: &PostOnly = &PostOnly;

    &[FILL_OR_KILL, POST_ONLY]
}

/// Policies that should be run after matching.
#[inline]
pub(super) const fn late_policies<'e, O, E>(
) -> &'e [&'e dyn Policy<O, E, seq::Late>]
where
    E: Exchange + ExchangeExt + 'e,
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
    const IMMEDIATE_OR_CANCEL: &ImmediateOrCancel = &ImmediateOrCancel;

    &[IMMEDIATE_OR_CANCEL]
}
