use std::collections::BTreeMap;
use std::ops::Deref;
use std::ops::DerefMut;

use exchange_core::Asset;

pub struct OrdersById<Order: Asset>(BTreeMap<<Order as Asset>::OrderId, Order>);

impl<Order: Asset> Default for OrdersById<Order> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Order: Asset> Deref for OrdersById<Order> {
    type Target = BTreeMap<<Order as Asset>::OrderId, Order>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Order: Asset> DerefMut for OrdersById<Order> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
