use compact_str::CompactString;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use thiserror::Error;

use crate::{Order, OrderId, OrderSide};

#[derive(Debug, Error)]
pub enum OrderRequestError {
    #[error("order type mismatch")]
    MismatchType,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type_op", rename_all = "UPPERCASE"))]
pub enum OrderRequest {
    Create {
        account_id: CompactString,
        amount: Decimal,
        order_id: CompactString,
        pair: CompactString,
        limit_price: Decimal,
        side: OrderSide,
    },
    Delete {
        order_id: CompactString,
    },
}

impl TryFrom<OrderRequest> for Order {
    type Error = OrderRequestError;

    #[inline]
    fn try_from(order_request: OrderRequest) -> Result<Self, Self::Error> {
        match order_request {
            OrderRequest::Create {
                order_id,
                amount,
                limit_price,
                side,
                ..
            } => Ok(Order::new_limit(
                OrderId::new(order_id.parse::<u64>().unwrap()),
                side,
                limit_price.trunc().to_u64().unwrap() * 100
                    + limit_price.fract().to_u64().unwrap(),
                amount.trunc().to_u64().unwrap() * 100
                    + amount.fract().to_u64().unwrap(),
            )),
            OrderRequest::Delete { .. } => Err(OrderRequestError::MismatchType),
        }
    }
}
