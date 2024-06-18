use std::fmt;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::ops::SubAssign;

macro_rules! forward_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        #[automatically_derived]
        impl $imp<$res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                (&self.0).$method(&other.0).into()
            }
        }

        #[automatically_derived]
        impl<'a> $imp<$res> for &'a $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                (&self.0).$method(&other.0).into()
            }
        }

        #[automatically_derived]
        impl<'b> $imp<&'b $res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &'b $res) -> $res {
                (&self.0).$method(&other.0).into()
            }
        }

        #[automatically_derived]
        impl<'a, 'b> $imp<&'b $res> for &'a $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &'b $res) -> $res {
                (&self.0).$method(&other.0).into()
            }
        }
    };
}

macro_rules! forward_binop_assign {
    (impl $imp:ident for $res:ty, $method:ident) => {
        #[automatically_derived]
        impl $imp<$res> for $res {
            #[inline]
            fn $method(&mut self, other: $res) {
                self.0.$method(&other.0)
            }
        }

        #[automatically_derived]
        impl<'a> $imp<&'a $res> for $res {
            #[inline]
            fn $method(&mut self, other: &'a $res) {
                self.0.$method(&other.0)
            }
        }
    };
}

macro_rules! amount {
    ($($t:ident)*) => ($(
        #[derive(Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
        #[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
        #[repr(transparent)]
        #[cfg_attr(feature = "serde", serde(transparent))]
        pub struct $t(::rust_decimal::Decimal);

        #[automatically_derived]
        impl $t {
            #[inline]
            pub fn is_zero(&self) -> bool {
                <$t as ::num::Zero>::is_zero(self)
            }
        }

        #[automatically_derived]
        impl<T> From<T> for $t
        where
            ::rust_decimal::Decimal: From<T>,
        {
            #[inline]
            fn from(decimal: T) -> $t {
                let decimal = ::rust_decimal::Decimal::from(decimal);
                Self(decimal)
            }
        }

        #[automatically_derived]
        impl fmt::Debug for $t {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        #[automatically_derived]
        impl fmt::Display for $t {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        #[automatically_derived]
        impl ::num::Zero for $t {
            #[inline]
            fn zero() -> Self {
                Self(<::rust_decimal::Decimal as ::num::Zero>::zero())
            }

            #[inline]
            fn is_zero(&self) -> bool {
                <::rust_decimal::Decimal as ::num::Zero>::is_zero(&self.0)
            }
        }

        forward_binop!(impl Add for $t, add);
        forward_binop!(impl Sub for $t, sub);
        forward_binop_assign!(impl AddAssign for $t, add_assign);
        forward_binop_assign!(impl SubAssign for $t, sub_assign);
    )*)
}

amount! { Notional Price Quantity }

impl Mul<Quantity> for Price {
    type Output = Notional;

    #[inline]
    fn mul(self, quantity: Quantity) -> Self::Output {
        let price = self;
        quantity * price
    }
}

impl Mul<Price> for Quantity {
    type Output = Notional;

    #[inline]
    fn mul(self, price: Price) -> Self::Output {
        let quantity = self;
        Notional(quantity.0 * price.0)
    }
}

impl Div<Price> for Notional {
    type Output = Quantity;

    #[inline]
    fn div(self, price: Price) -> Self::Output {
        let notional = self;
        Quantity(notional.0 / price.0)
    }
}

impl Div<Quantity> for Notional {
    type Output = Price;

    #[inline]
    fn div(self, quantity: Quantity) -> Self::Output {
        let notional = self;
        Price(notional.0 / quantity.0)
    }
}
