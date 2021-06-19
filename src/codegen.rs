macro_rules! generate_code {
    {
        $(
            $field:ident : $type:ident { $($variant:ident $({ $($var_field:ident : $var_type:ty),+ })? => $lit:literal),+ $(,)? }
        )+
    } => {
        $(
            #[derive(Debug, Copy, Clone, PartialEq)]
            pub enum $type {
                $(
                    $variant $({ $($var_field: $var_type),+ })?
                ),+
            }
            generate_code!(@enum_try_from $type [] [$(($lit $variant $({$($var_field)+})?))+]);
        )+
        generate_code!(@spec_struct $($field $type)+);
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
    (@spec_struct $($field:ident $type:ty)+) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct Specifier {
            $(
                pub $field: $type
            ),+
        }
    };
    (@fn_format_value $($dim:tt)+) => {
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
