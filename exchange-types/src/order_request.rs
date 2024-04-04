use compact_str::CompactString;
use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

use crate::Order;
use crate::OrderId;
use crate::OrderSide;
use crate::OrderType;
use crate::TimeInForce;

#[derive(Debug, Error)]
pub enum OrderRequestError {
    #[error("order type mismatch")]
    MismatchType,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type_op", rename_all = "UPPERCASE"))]
pub enum OrderRequest {
    Create {
        account_id: Uuid,
        amount: Decimal,
        order_id: Uuid,
        pair: CompactString,
        limit_price: Decimal,
        side: OrderSide,
    },
    Delete {
        order_id: Uuid,
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
            } => Ok(Order::new(
                OrderId::new(order_id),
                side,
                OrderType::Limit {
                    limit_price,
                    time_in_force: TimeInForce::default(),
                    amount,
                    filled: Decimal::ZERO,
                },
            )),
            OrderRequest::Delete { .. } => Err(OrderRequestError::MismatchType),
        }
    }
}
