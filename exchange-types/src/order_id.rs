use uuid::Uuid;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderId(Uuid);

impl OrderId {
    #[inline]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[inline]
    #[cfg(any(test, feature = "test"))]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<Uuid> for OrderId {
    #[inline]
    fn from(uuid: Uuid) -> Self {
        Self::new(uuid)
    }
}

impl From<&Uuid> for OrderId {
    #[inline]
    fn from(uuid: &Uuid) -> Self {
        Self::new(*uuid)
    }
}

#[cfg(feature = "rand")]
mod __rand {
    use rand::distributions::Standard;
    use rand::prelude::*;

    use super::*;

    impl Distribution<OrderId> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> OrderId {
            let uuid = Uuid::from_bytes(rng.gen());
            OrderId::from(uuid)
        }
    }
}
