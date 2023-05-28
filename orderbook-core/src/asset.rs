pub trait Asset<Order = Self>: Ord {
    /// Order amount.
    type OrderAmount: Copy + Ord;
    /// Order unique identifier.
    type OrderId: Copy + Eq;
    /// Order price.
    type OrderPrice: Copy + Ord;
    /// Order side.
    type OrderSide: Opposite;
    /// Order current status.
    type OrderStatus: Copy + Eq;
    /// Trade struct.
    type Trade;
    /// Return order unique identifier.
    fn id(&self) -> Self::OrderId;
    /// Return order side.
    fn side(&self) -> Self::OrderSide;
    /// Return order limit price.
    fn limit_price(&self) -> Self::OrderPrice;
    /// Return order remaining amount.
    fn remaining(&self) -> Self::OrderAmount;
    /// Return current order status.
    fn status(&self) -> Self::OrderStatus;
    fn is_closed(&self) -> bool;
    fn is_immediate_or_cancel(&self) -> bool;
    fn is_post_only(&self) -> bool;
    fn trade(&mut self, order: &mut Order) -> Option<Self::Trade>;
    fn matches(&self, order: &Order) -> bool;
    fn cancel(&mut self);
}

pub trait Opposite<Opposite = Self> {
    fn opposite(&self) -> Opposite;
}
