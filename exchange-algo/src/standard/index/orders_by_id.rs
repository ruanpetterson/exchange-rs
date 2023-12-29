use std::collections::BTreeMap;
use std::path::Path;

use either::Either;
use exchange_types::{Order, OrderId};
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use tap::Pipe as _;

use crate::standard::Error;
use crate::standard::{DB_CORRUPT_MSG, DB_DEAD_MSG};

pub struct OrdersById {
    persistence: Persistence,
}

enum Persistence {
    InMemory(BTreeMap<OrderId, Vec<u8>>),
    Sled(sled::Tree),
}

struct CustomSerializer;

impl CustomSerializer {
    #[inline]
    fn from_slice<'de, T: Deserialize<'de>>(
        bytes: &'de [u8],
    ) -> Result<T, Error> {
        postcard::from_bytes(bytes).map_err(From::from)
    }

    #[inline]
    fn to_vec<T: Serialize>(value: &T) -> Result<heapless::Vec<u8, 64>, Error> {
        postcard::to_vec(value).map_err(From::from)
    }
}

impl Default for OrdersById {
    #[inline]
    fn default() -> Self {
        Self {
            persistence: Persistence::InMemory(Default::default()),
        }
    }
}

impl OrdersById {
    #[inline]
    pub fn use_sled(path: impl AsRef<Path>) -> Result<Self, Error> {
        let tree = sled::Config::default()
            .path(path)
            .mode(sled::Mode::HighThroughput)
            .open()?
            .open_tree("orders_by_id")
            .expect(DB_DEAD_MSG);

        Self {
            persistence: Persistence::Sled(tree),
        }
        .pipe(Ok)
    }
}

impl OrdersById {
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = Order> + '_ {
        match &self.persistence {
            Persistence::InMemory(map) => map
                .values()
                .map(|serialized_order| {
                    CustomSerializer::from_slice::<Order>(serialized_order)
                        .expect(DB_CORRUPT_MSG)
                })
                .pipe(Either::Left),
            Persistence::Sled(tree) => tree
                .into_iter()
                .map_ok(|(_, serialized_order)| {
                    CustomSerializer::from_slice::<Order>(&serialized_order)
                        .expect(DB_CORRUPT_MSG)
                })
                .map(|order| order.expect(DB_DEAD_MSG))
                .pipe(Either::Right),
        }
    }

    #[inline]
    pub fn get(&self, order_id: &OrderId) -> Option<Order> {
        match &self.persistence {
            Persistence::InMemory(map) => map.get(order_id).map(Either::Left),
            Persistence::Sled(tree) => {
                tree.get(order_id).expect(DB_DEAD_MSG).map(Either::Right)
            }
        }
        .map(|serialized_order| {
            let bytes = AsRef::<[u8]>::as_ref(&serialized_order);
            CustomSerializer::from_slice(bytes).expect(DB_CORRUPT_MSG)
        })
    }

    #[inline]
    pub fn insert(&mut self, order_id: OrderId, order: &Order) {
        let serialized_order = CustomSerializer::to_vec(order)
            .expect("serialization should never fail");

        match &mut self.persistence {
            Persistence::InMemory(map) => {
                map.insert(order_id, serialized_order.to_vec());
            }
            Persistence::Sled(tree) => {
                tree.insert(order_id, &*serialized_order)
                    .expect(DB_DEAD_MSG);
            }
        }
    }

    #[inline]
    pub fn remove(&mut self, order_id: &OrderId) -> Option<Order> {
        match &mut self.persistence {
            Persistence::InMemory(map) => {
                map.remove(order_id).map(Either::Left)
            }
            Persistence::Sled(tree) => {
                tree.remove(order_id).expect(DB_DEAD_MSG).map(Either::Right)
            }
        }
        .map(|serialized_order| {
            let bytes = AsRef::<[u8]>::as_ref(&serialized_order);
            CustomSerializer::from_slice(bytes).expect(DB_CORRUPT_MSG)
        })
    }
}
