use std::fmt;

use crate::Specifier;

pub trait FormattableValue {
    fn supports_format(&self, specifier: &Specifier) -> bool;
    fn fmt_display(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn fmt_debug(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn fmt_octal(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn fmt_lower_hex(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn fmt_upper_hex(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn fmt_binary(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn fmt_lower_exp(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn fmt_upper_exp(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

pub struct ValueFormatter<'v, V: FormattableValue>(pub &'v V);

impl<'v, V: FormattableValue> fmt::Display for ValueFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_display(f)
    }
}

impl<'v, V: FormattableValue> fmt::Debug for ValueFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_debug(f)
    }
}

impl<'v, V: FormattableValue> fmt::Octal for ValueFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_octal(f)
    }
}

impl<'v, V: FormattableValue> fmt::LowerHex for ValueFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_lower_hex(f)
    }
}

impl<'v, V: FormattableValue> fmt::UpperHex for ValueFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_upper_hex(f)
    }
}

impl<'v, V: FormattableValue> fmt::Binary for ValueFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_binary(f)
    }
}

impl<'v, V: FormattableValue> fmt::LowerExp for ValueFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_lower_exp(f)
    }
}

impl<'v, V: FormattableValue> fmt::UpperExp for ValueFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_upper_exp(f)
    }
}
