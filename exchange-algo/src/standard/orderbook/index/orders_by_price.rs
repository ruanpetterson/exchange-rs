use std::collections::{BTreeMap, VecDeque};
use std::ops::{Deref, DerefMut};

use exchange_core::Asset;

pub struct OrdersByPrice<Order: Asset>(
    BTreeMap<<Order as Asset>::OrderPrice, VecDeque<<Order as Asset>::OrderId>>,
);

impl<Order: Asset> Default for OrdersByPrice<Order> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Order: Asset> Deref for OrdersByPrice<Order> {
    type Target = BTreeMap<
        <Order as Asset>::OrderPrice,
        VecDeque<<Order as Asset>::OrderId>,
    >;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Order: Asset> DerefMut for OrdersByPrice<Order> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
