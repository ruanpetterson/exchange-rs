use std::ops::Deref;
use std::ops::DerefMut;

use exchange_core::Asset;
use slab::Slab;

pub struct OrdersById<Order: Asset>(Slab<Order>);

impl<Order: Asset> OrdersById<Order> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Slab::with_capacity(capacity))
    }
}

impl<Order: Asset> Default for OrdersById<Order> {
    #[inline]
    fn default() -> Self {
        Self(Slab::default())
    }
}

impl<Order: Asset> Deref for OrdersById<Order> {
    type Target = Slab<Order>;

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
