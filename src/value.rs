//! Defines traits and types to help make arbitrary values formattable at runtime.

use std::fmt;

use crate::Specifier;

/// A type that indicates whether its value supports a specific format, and provides formatting
/// functions that correspond to different format types.
pub trait FormattableValue {
    /// Returns `true` if `self` can be formatted using the given specifier.
    fn supports_format(&self, specifier: &Specifier) -> bool;
    /// Formats the value the way it would be formatted if it implemented `std::fmt::Display`.
    fn fmt_display(&self, f: &mut fmt::Formatter) -> fmt::Result;
    /// Formats the value the way it would be formatted if it implemented `std::fmt::Debug`.
    fn fmt_debug(&self, f: &mut fmt::Formatter) -> fmt::Result;
    /// Formats the value the way it would be formatted if it implemented `std::fmt::Octal`.
    fn fmt_octal(&self, f: &mut fmt::Formatter) -> fmt::Result;
    /// Formats the value the way it would be formatted if it implemented `std::fmt::LowerHex`.
    fn fmt_lower_hex(&self, f: &mut fmt::Formatter) -> fmt::Result;
    /// Formats the value the way it would be formatted if it implemented `std::fmt::UpperHex`.
    fn fmt_upper_hex(&self, f: &mut fmt::Formatter) -> fmt::Result;
    /// Formats the value the way it would be formatted if it implemented `std::fmt::Binary`.
    fn fmt_binary(&self, f: &mut fmt::Formatter) -> fmt::Result;
    /// Formats the value the way it would be formatted if it implemented `std::fmt::LowerExp`.
    fn fmt_lower_exp(&self, f: &mut fmt::Formatter) -> fmt::Result;
    /// Formats the value the way it would be formatted if it implemented `std::fmt::UpperExp`.
    fn fmt_upper_exp(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

/// Holds a `FormattableValue` and implements all the `std::fmt` formatting traits.
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
