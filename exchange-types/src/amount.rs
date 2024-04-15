use std::fmt;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Sub;
use std::ops::SubAssign;

macro_rules! forward_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl $imp<$res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                (&self.0).$method(&other.0).into()
            }
        }

        impl<'a> $imp<$res> for &'a $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                (&self.0).$method(&other.0).into()
            }
        }

        impl<'b> $imp<&'b $res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &'b $res) -> $res {
                (&self.0).$method(&other.0).into()
            }
        }

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
        impl $imp<$res> for $res {
            #[inline]
            fn $method(&mut self, other: $res) {
                self.0.$method(&other.0)
            }
        }

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

        impl From<::rust_decimal::Decimal> for $t {
            #[inline]
            fn from(decimal: ::rust_decimal::Decimal) -> $t {
                Self(decimal)
            }
        }

        impl fmt::Debug for $t {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl fmt::Display for $t {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl Deref for $t {
            type Target = ::rust_decimal::Decimal;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $t {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl ::num::Zero for $t {
            fn zero() -> Self {
                Self(<::rust_decimal::Decimal as ::num::Zero>::zero())
            }

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

amount! { Amount Price }