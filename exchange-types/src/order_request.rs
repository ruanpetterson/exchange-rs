use compact_str::CompactString;
use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

use crate::order_type::ByBase;
use crate::Order;
use crate::OrderId;
use crate::OrderSide;
use crate::OrderType;
use crate::Price;
use crate::Quantity;
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
        amount: Quantity,
        order_id: Uuid,
        symbol: CompactString,
        limit_price: Price,
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
                    priced_by: ByBase {
                        quantity: amount,
                        filled: Decimal::ZERO.into(),
                    },
                },
            )),
            OrderRequest::Delete { .. } => Err(OrderRequestError::MismatchType),
        }
    }
}
