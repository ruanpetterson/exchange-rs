use exchange_core::Asset;
use exchange_core::Exchange;
use exchange_core::Trade;

use super::seq;
use super::Policy;

pub(super) struct ImmediateOrCancel;
impl<O, E> Policy<O, E, seq::Late> for ImmediateOrCancel
where
    E: Exchange,
    <E as Exchange>::Order: Trade<O>,
    O: Asset<
        OrderAmount = <<E as Exchange>::Order as Asset>::OrderAmount,
        OrderId = <<E as Exchange>::Order as Asset>::OrderId,
        OrderPrice = <<E as Exchange>::Order as Asset>::OrderPrice,
        OrderSide = <<E as Exchange>::Order as Asset>::OrderSide,
        OrderStatus = <<E as Exchange>::Order as Asset>::OrderStatus,
    >,
{
    #[inline]
    fn enforce(incoming_order: &mut O, _: &E) {
        if incoming_order.is_immediate_or_cancel() {
            // If incoming order is immediate or cancel, it must be closed
            // at the end of matching.
            incoming_order.cancel();
        }
    }
}
