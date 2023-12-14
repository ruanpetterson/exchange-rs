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

impl AsRef<[u8]> for OrderId {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
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
