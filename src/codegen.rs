//! The `generate_code!` macro enerates the `Specifier` struct, `format_value` function, and all the
//! code they need.
//! 
//! The macro expects definitions of "dimensions" of the format specifier (e.g. width, precision,
//! and format to use). Each dimension has to define the name of the field to generate in the
//! `Specifier` struct, the name of the enum type to generate for that field, and the definition of
//! each variant for that enum. Each variant definition declares the variant name, optionally with
//! one or more fields for that variant to contain, and then the format string fragment to generate
//! when that variant is matched.
//! 
//! The way `format_value` function works is through a tree of nested `match` blooks on `Specifier` 
//! fields, with a call to `write!` macro with a different formatting string at each leaf.
//! 
//! # Examples
//! ```ignore
//! generate_code! {
//!     foo: Foo {
//!         Argle => "",
//!         Bargle { glop_glyf: usize } => "glop_glyf$",
//!     }
//! 
//!     bar: Bar {
//!         Olle => "",
//!         Bolle => "@",
//!     }
//! }
//! ```
//! 
//! The resulting `format_value` would look like this:
//! ```ignore
//! pub fn format_value<V>(specifier: &Specifier, value: &V, f: &mut fmt::Formatter) -> fmt::Result {
//!     match (specifier.foo) {
//!         Argle => match (specifier.bar) {
//!             Olle => write!(f, "{:}", value),
//!             Bolle => write!(f, "{:@}", value),
//!         },
//!         Bargle { glop_glyf } => match (specifier.bar) {
//!             Olle => write!(f, "{:glop_glyf$}", value, glop_glyf),
//!             Bolle => write!(f, "{:glop_glyf$@}", value, glop_glyf),
//!         }
//!     }
//! }

macro_rules! generate_code {
    {
        $(
            $(#[$dim_meta:meta])*
            $field:ident : $type:ident {
                $(
                    $variant:ident $({ $($var_field:ident : $var_type:ty),+ })? => $lit:literal
                ),+ $(,)? 
            }
        )+
    } => {
        $(
            $(#[$dim_meta])*
            #[derive(Debug, Copy, Clone, PartialEq)]
            #[allow(missing_docs)]
            pub enum $type {
                $(
                    $variant $({ $($var_field: $var_type),+ })?
                ),+
            }
            generate_code!(@enum_try_from $type [] [$(($lit $variant $({$($var_field)+})?))+]);
        )+

        /// The specification for the format of an argument in the formatting string.
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct Specifier {
            $(
                $(#[$dim_meta])*
                pub $field: $type
            ),+
        }

        generate_code!(@fn_format_value
            $(
                [$field $type $([$lit $variant $([$($var_field)+])?])+]
            )+
        );
    };
    (@enum_try_from
        $type:ident [$($munched:tt)*] [($lit:literal $variant:ident) $($tail:tt)*]
    ) => {
        generate_code!(@enum_try_from $type [$($munched)* ($lit $variant)] [$($tail)*]);
    };
    (@enum_try_from
        $type:ident [$($munched:tt)*] [($lit:literal $variant:ident $_:tt) $($tail:tt)*]
    ) => {
    };
    (@enum_try_from
        $type:ident [$(($lit:literal $variant:ident))+] []
    ) => {
        impl TryFrom<&str> for $type {
            type Error = ();
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value.as_ref() {
                    $($lit => Ok($type::$variant),)+
                    _ => Err(())
                }
            }
        }
    };
    (@fn_format_value $($dim:tt)+) => {
        /// Formats the given value using the given formatter and the given format specification.
        /// 
        /// Since the implementation of `format_value` employs the `write!` macro, the `value` must
        /// implement all of the `std::fmt` formatting traits. Which trait will actually be used is
        /// determined at runtime, based on the contents of the `specifier`.
        pub fn format_value<V>(specifier: &Specifier, value: &V, f: &mut fmt::Formatter) -> fmt::Result
        where
            V: fmt::Display
                + fmt::Debug
                + fmt::Octal
                + fmt::LowerHex
                + fmt::UpperHex
                + fmt::Binary
                + fmt::LowerExp
                + fmt::UpperExp,
        {
            generate_code!(@matcher (specifier, value, f, "", []) $($dim)+)
        }
    };
    (@matcher ($spec:ident, $val:ident, $out:ident, $prefix:expr, $named_args:tt) $head_dim:tt $($tail_dim:tt)+) => {
        generate_code!(@matcher_branch ($spec, $val, $out, $prefix, $named_args) $head_dim [$($tail_dim)+])
    };
    (@matcher ($spec:ident, $val:ident, $out:ident, $prefix:expr, $named_args:tt) $last_dim:tt) => {
        generate_code!(@matcher_leaf ($spec, $val, $out, $prefix, $named_args) $last_dim)
    };
    (@matcher_branch
        ($spec:ident, $val:ident, $out:ident, $prefix:expr, $named_args:tt)
        [$field:ident $type:ident $([$lit:literal $variant:ident $([$($var_field:ident)+])?])+]
        $tail:tt
    ) => {
        match $spec.$field {
            $(
                $type::$variant $({ $($var_field),+ })? => generate_code!(
                    @matcher_tail
                    ($spec, $val, $out, concat!($prefix, $lit))
                    $named_args
                    [$($($var_field)+)?]
                    $tail
                )
            ),+
        }
    };
    (@matcher_tail ($spec:ident, $val:ident, $out:ident, $prefix:expr) [$($lhs_arg:ident)*] [$($rhs_arg:ident)*] [$($dim:tt)+]) => {
        generate_code!(@matcher ($spec, $val, $out, $prefix, [$($lhs_arg)* $($rhs_arg)*]) $($dim)+)
    };
    (@matcher_leaf
        ($spec:ident, $val:ident, $out:ident, $prefix:expr, $named_args:tt)
        [$field:ident $type:ident $([$lit:literal $variant:ident $([$($var_field:ident)+])?])+]
    ) => {
        match $spec.$field {
            $(
                $type::$variant $({ $($var_field),+ })? => generate_code!(
                    @matcher_concat_args
                    ($spec, $val, $out, concat!($prefix, $lit))
                    $named_args
                    [$($($var_field)+)?]
                )
            ),+
        }
    };
    (@matcher_concat_args ($spec:ident, $val:ident, $out:ident, $format_str:expr) [$($lhs_arg:ident)*] [$($rhs_arg:ident)*]) => {
        generate_code!(@matcher_write ($spec, $val, $out, $format_str) [$($lhs_arg)* $($rhs_arg)*])
    };
    (@matcher_write ($spec:ident, $val:ident, $out:ident, $format_str:expr) [$($named_arg:ident)*]) => {
        write!(
            $out,
            concat!("{:", $format_str, "}"),
            $val,
            $($named_arg = $named_arg),*
        )
    };
}
