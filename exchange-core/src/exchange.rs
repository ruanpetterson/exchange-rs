use crate::Asset;

pub trait Exchange<Rhs = Self>: Asset
where
    Rhs: Asset,
{
    /// Returns `Ok` if orders match.
    fn matches(&self, other: &Rhs) -> Result<(), Self::TradeError>;

    /// Execute a trade.
    fn trade(
        &mut self,
        other: &mut Rhs,
    ) -> Result<Self::Trade, Self::TradeError>;
}
