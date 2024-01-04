use exchange_core::{Asset, Tree};

use super::Policy;

pub(super) struct ImmediateOrCancel;
impl<E: Tree> Policy<E> for ImmediateOrCancel {
    #[inline]
    fn enforce(incoming_order: &mut E::Order, _: &E) {
        if incoming_order.is_immediate_or_cancel() {
            // If incoming order is immediate or cancel, it must be closed
            // at the end of matching.
            incoming_order.cancel();
        }
    }
}
