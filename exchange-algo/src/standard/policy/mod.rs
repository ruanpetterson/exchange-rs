mod fill_or_kill;
mod immediate_or_cancel;
mod post_only;

use exchange_core::{Asset, Exchange, ExchangeExt};

pub(crate) trait Policy<A: Asset, E: Exchange> {
    fn enforce(&self, order: &mut A, exchange: &E);
}

/// Policies that should be run before matching.
pub(super) const fn before_policies<'e, E: Exchange + ExchangeExt + 'e>(
) -> &'e [&'e dyn Policy<E::Order, E>] {
    &[&fill_or_kill::FillOrKill, &post_only::PostOnly]
}

/// Policies that should be run after matching.
pub(super) const fn late_policies<'e, E: Exchange + ExchangeExt + 'e>(
) -> &'e [&'e dyn Policy<E::Order, E>] {
    &[&immediate_or_cancel::ImmediateOrCancel]
}
