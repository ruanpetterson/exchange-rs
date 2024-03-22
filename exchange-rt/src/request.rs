use std::fmt;

use compact_str::CompactString;
use exchange_core::Asset;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("order type mismatch")]
    MismatchType,
}

pub enum Request<O: Asset> {
    Create {
        pair: CompactString,
        order: O,
    },
    Delete {
        /// Order unique identifier.
        order_id: <O as Asset>::OrderId,
    },
}

impl<O: Asset> Request<O> {
    #[inline]
    pub fn into_inner(self) -> Result<O, RequestError> {
        match self {
            Request::Create { order, .. } => Ok(order),
            Request::Delete { .. } => Err(RequestError::MismatchType),
        }
    }
}

impl<O> fmt::Debug for Request<O>
where
    O: Asset + fmt::Debug,
    <O as Asset>::OrderId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("Request");
        match self {
            Request::Create { pair, order } => {
                f.field("pair", pair).field("order", order)
            }
            Request::Delete { order_id } => f.field("order_id", order_id),
        }
        .finish()
    }
}

#[cfg(feature = "serde")]
mod __serde {
    use std::marker::PhantomData;

    use serde::ser::SerializeMap as _;
    use serde::{de, ser};

    use super::*;

    impl<'de, O> de::Deserialize<'de> for Request<O>
    where
        O: Asset + 'de,
        O: de::Deserialize<'de>,
        <O as Asset>::OrderId: de::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            struct Visitor<'de, O>
            where
                O: Asset + de::Deserialize<'de>,
                <O as Asset>::OrderId: de::Deserialize<'de>,
            {
                marker: PhantomData<&'de O>,
            }

            impl<'de, O> de::Visitor<'de> for Visitor<'de, O>
            where
                O: Asset + de::Deserialize<'de>,
                <O as Asset>::OrderId: de::Deserialize<'de>,
            {
                type Value = Request<O>;

                #[inline]
                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("struct OrderRequest")
                }

                #[inline]
                fn visit_map<A>(
                    self,
                    mut map: A,
                ) -> Result<Self::Value, A::Error>
                where
                    A: de::MapAccess<'de>,
                {
                    let type_op = match map.next_key::<&str>()? {
                        Some("type_op") => map.next_value::<&str>()?,
                        _ => {
                            return Err(de::Error::missing_field("type_op"));
                        }
                    };

                    let request = match type_op {
                        "CREATE" => {
                            let pair = match map.next_key::<&str>()? {
                                Some("pair") => map.next_value::<&str>()?,
                                _ => {
                                    return Err(de::Error::missing_field(
                                        "pair",
                                    ));
                                }
                            };

                            let mut rest = std::iter::from_fn(|| {
                                map.next_entry().ok().flatten()
                            })
                            .fuse()
                            .map(Some)
                            .collect::<Vec<_>>();

                            let order = O::deserialize(
                                serde::__private::de::FlatMapDeserializer(
                                    &mut rest,
                                    std::marker::PhantomData,
                                ),
                            )?;

                            Request::Create {
                                pair: CompactString::new(pair),
                                order,
                            }
                        }
                        "DELETE" => {
                            let mut order_id = None;

                            while let Some(key) = map.next_key::<&str>()? {
                                if key != "order_id" {
                                    continue;
                                }

                                let value = map.next_value()?;
                                let _ = order_id.replace(value);
                            }

                            let Some(order_id) = order_id else {
                                return Err(de::Error::missing_field(
                                    "order_id",
                                ));
                            };

                            Request::Delete { order_id }
                        }
                        type_op => {
                            return Err(de::Error::invalid_value(
                                de::Unexpected::Str(type_op),
                                &"CREATE or DELETE",
                            ))
                        }
                    };

                    Ok(request)
                }
            }

            deserializer.deserialize_struct(
                "OrderRequest",
                &[],
                Visitor {
                    marker: PhantomData,
                },
            )
        }
    }

    impl<O> ser::Serialize for Request<O>
    where
        O: Asset + ser::Serialize,
        <O as Asset>::OrderId: ser::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            match self {
                Request::Create { pair, order } => {
                    let mut map = serializer.serialize_map(Some(1))?;
                    map.serialize_entry("type_op", "CREATE")?;
                    map.serialize_entry("pair", &**pair)?;
                    ser::Serialize::serialize(
                        &order,
                        serde::__private::ser::FlatMapSerializer(&mut map),
                    )?;
                    map.end()
                }
                Request::Delete { order_id } => {
                    let mut map = serializer.serialize_map(Some(1))?;
                    map.serialize_entry("type_op", "DELETE")?;
                    map.serialize_entry("order_id", order_id)?;
                    map.end()
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use exchange_types::{Order, OrderSide};

        use super::*;

        #[test]
        fn it_works() {
            let order = Order::builder()
                .side(OrderSide::Ask)
                .market(rust_decimal::Decimal::new(10, 0))
                .build();

            eprintln!("{order:?}");

            let to_serialize_request = Request::Create {
                pair: "BTC/USDC".into(),
                order,
            };

            eprintln!("{to_serialize_request:?}");

            let serialized_request =
                serde_json::to_string_pretty(&to_serialize_request).unwrap();

            eprintln!("{serialized_request}");

            let deserialized_request =
                serde_json::from_str::<Request<Order>>(&serialized_request)
                    .unwrap();

            eprintln!("{deserialized_request:?}");
        }
    }
}
