macro_rules! enum_with_repr {
    {
        $( #[$attr:meta] )*
        $vis:vis enum $ty:ident : $repr:ty {
            $(
                $( #[$var_attr:meta] )*
                $var:ident = $val:literal,
            )*
        }
    } => {
        $( #[$attr] )*
        #[repr($repr)]
        $vis enum $ty {
            $(
                $( #[$var_attr] )*
                $var = $val,
            )*
        }

        impl From<$ty> for $repr {
            #[inline]
            fn from(val: $ty) -> Self {
                val as _
            }
        }

        impl TryFrom<$repr> for $ty {
            type Error = $repr;

            #[inline]
            fn try_from(val: $repr) -> Result<Self, Self::Error> {
                match val {
                    $( $val => Ok(Self::$var), )*
                    _ => Err(val),
                }
            }
        }
    }
}

pub(crate) use enum_with_repr;
