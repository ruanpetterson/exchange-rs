mod fill_or_kill;
mod immediate_or_cancel;
mod post_only;

use exchange_core::{Exchange, ExchangeExt};

use self::fill_or_kill::FillOrKill;
use self::immediate_or_cancel::ImmediateOrCancel;
use self::post_only::PostOnly;

pub(crate) trait Policy<E: Exchange> {
    fn enforce(order: &mut <E as Exchange>::IncomingOrder, exchange: &E);
}

/// Policies that should be run before matching.
#[inline]
pub(super) const fn before_policies<'e, E: Exchange + ExchangeExt + 'e>(
) -> &'e [fn(&mut <E as Exchange>::IncomingOrder, &E)] {
    &[
        <FillOrKill as Policy<E>>::enforce,
        <PostOnly as Policy<E>>::enforce,
    ]
}

/// Policies that should be run after matching.
#[inline]
pub(super) const fn late_policies<'e, E: Exchange + ExchangeExt + 'e>(
) -> &'e [fn(&mut <E as Exchange>::IncomingOrder, &E)] {
    &[<ImmediateOrCancel as Policy<E>>::enforce]
}
