macro_rules! generate_code {
    {
        $(
            $field:ident : $type:ident { $($lit:literal => $variant:ident $({ $($var_field:ident : $var_type:ty),+ })?),+ $(,)? }
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
        generate_code!(@formatting_traits [] $([$($variant)+])+);
        generate_code!(@spec_struct $($field $type)+);
        generate_code!(@arg_struct
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
    (@formatting_traits [$($munched_dim:tt)*] $head_dim:tt $($tail_dim:tt)+) => {
        generate_code!(@formatting_traits [$($munched_dim)* $head_dim] $($tail_dim)+);
    };
    (@formatting_traits [$($dim:tt)+] [$($variant:ident)+]) => {
        pub trait FormattableValue {
            fn supports_format(&self, specifier: &Specifier) -> bool;
            paste! {
                $(
                    fn [<fmt_ $variant:snake>](&self, f: &mut fmt::Formatter) -> fmt::Result;
                )+
            }
        }

        struct ValueFormatter<'v, V: FormattableValue>(&'v V);

        $(
            impl<'v, V: FormattableValue> fmt::$variant for ValueFormatter<'v, V> {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    paste! {
                        self.0.[<fmt_ $variant:snake>](f)
                    }
                }
            }
        )+
    };
    (@spec_struct $($field:ident $type:ty)+) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct Specifier {
            $(
                pub $field: $type
            ),+
        }
    };
    (@arg_struct $($dim:tt)+) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct Argument<'v, V: FormattableValue> {
            pub specifier: Specifier, 
            pub value: &'v V, 
            _private: ()
        }

        impl<'v, V: FormattableValue> Argument<'v, V> {
            pub fn new(specifier: Specifier, value: &'v V) -> Result<Argument<'v, V>, ()> {
                if value.supports_format(&specifier) {
                    Ok(Argument { specifier, value, _private: () })
                } else {
                    Err(())
                }
            }
        }

        impl<'v, V: FormattableValue> fmt::Display for Argument<'v, V> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                generate_code!(@matcher (self, f, "", []) $($dim)+)
            }
        }
    };
    (@matcher ($self:ident, $out:ident, $prefix:expr, $named_args:tt) $head_dim:tt $($tail_dim:tt)+) => {
        generate_code!(@matcher_branch ($self, $out, $prefix, $named_args) $head_dim [$($tail_dim)+])
    };
    (@matcher ($self:ident, $out:ident, $prefix:expr, $named_args:tt) $last_dim:tt) => {
        generate_code!(@matcher_leaf ($self, $out, $prefix, $named_args) $last_dim)
    };
    (@matcher_branch
        ($self:ident, $out:ident, $prefix:expr, $named_args:tt)
        [$field:ident $type:ident $([$lit:literal $variant:ident $([$($var_field:ident)+])?])+]
        $tail:tt
    ) => {
        match $self.specifier.$field {
            $(
                $type::$variant $({ $($var_field),+ })? => generate_code!(
                    @matcher_tail
                    ($self, $out, concat!($prefix, $lit))
                    $named_args
                    [$($($var_field)+)?]
                    $tail
                )
            ),+
        }
    };
    (@matcher_tail ($self:ident, $out:ident, $prefix:expr) [$($lhs_arg:ident)*] [$($rhs_arg:ident)*] [$($dim:tt)+]) => {
        generate_code!(@matcher ($self, $out, $prefix, [$($lhs_arg)* $($rhs_arg)*]) $($dim)+)
    };
    (@matcher_leaf
        ($self:ident, $out:ident, $prefix:expr, $named_args:tt)
        [$field:ident $type:ident $([$lit:literal $variant:ident $([$($var_field:ident)+])?])+]
    ) => {
        match $self.specifier.$field {
            $(
                $type::$variant $({ $($var_field),+ })? => generate_code!(
                    @matcher_concat_args
                    ($self, $out, concat!($prefix, $lit))
                    $named_args
                    [$($($var_field)+)?]
                )
            ),+
        }
    };
    (@matcher_concat_args ($self:ident, $out:ident, $format_str:expr) [$($lhs_arg:ident)*] [$($rhs_arg:ident)*]) => {
        generate_code!(@matcher_write ($self, $out, $format_str) [$($lhs_arg)* $($rhs_arg)*])
    };
    (@matcher_write ($self:ident, $out:ident, $format_str:expr) [$($named_arg:ident)*]) => {
        write!(
            $out,
            concat!("{:", $format_str, "}"),
            ValueFormatter($self.value),
            $($named_arg = $named_arg),*
        )
    };
}
