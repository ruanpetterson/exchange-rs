use exchange_core::Opposite;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "UPPERCASE"))]
pub enum OrderSide {
    #[cfg_attr(feature = "serde", serde(alias = "SELL"))]
    Ask,
    #[cfg_attr(feature = "serde", serde(alias = "BUY"))]
    Bid,
}

impl Opposite for OrderSide {
    #[inline]
    fn opposite(&self) -> Self {
        match self {
            OrderSide::Ask => OrderSide::Bid,
            OrderSide::Bid => OrderSide::Ask,
        }
    }
}

#[cfg(feature = "rand")]
mod __rand {
    use rand::distributions::Standard;
    use rand::prelude::*;

    use super::*;

    impl Distribution<OrderSide> for Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> OrderSide {
            match rng.gen_bool(0.5) {
                true => OrderSide::Ask,
                false => OrderSide::Bid,
            }
        }
    }
}
