use crate::{Asset, Exchange};

pub trait Policy<A: Asset, E: Exchange> {
    fn enforce(&self, order: &mut A, exchange: &E);
}
