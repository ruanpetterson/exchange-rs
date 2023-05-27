pub trait Asset<Order = Self>: Ord + Eq {
    /// Order unique identifier.
    type OrderId: Copy + Clone + Eq;
    /// Order current status.
    type OrderStatus: Eq + Copy + Clone;
    /// Order side.
    type OrderSide: Opposite;
    /// Trade struct.
    type Trade;
    /// Return order unique identifier.
    fn id(&self) -> Self::OrderId;
    /// Return order side.
    fn side(&self) -> Self::OrderSide;
    /// Return order limit price.
    fn limit_price(&self) -> u64;
    /// Return order remaining amount.
    fn remaining(&self) -> u64;
    /// Return current order status.
    fn status(&self) -> Self::OrderStatus;
    fn is_closed(&self) -> bool;
    fn trade(&mut self, order: &mut Order) -> Option<Self::Trade>;
    fn cancel(&mut self);
}

pub trait Opposite<Opposite = Self> {
    fn opposite(&self) -> Opposite;
}
