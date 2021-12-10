//! Defines traits and types to help make arbitrary values formattable at runtime.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use crate::Specifier;

/// A type that indicates whether its value supports a specific format, and provides formatting
/// functions that correspond to different format types.
pub trait FormatArgument {
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

/// Holds a `FormatArgument` and implements all the `std::fmt` formatting traits.
pub struct ArgumentFormatter<'v, V: FormatArgument>(pub &'v V);

impl<'v, V: FormatArgument> fmt::Display for ArgumentFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_display(f)
    }
}

impl<'v, V: FormatArgument> fmt::Debug for ArgumentFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_debug(f)
    }
}

impl<'v, V: FormatArgument> fmt::Octal for ArgumentFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_octal(f)
    }
}

impl<'v, V: FormatArgument> fmt::LowerHex for ArgumentFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_lower_hex(f)
    }
}

impl<'v, V: FormatArgument> fmt::UpperHex for ArgumentFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_upper_hex(f)
    }
}

impl<'v, V: FormatArgument> fmt::Binary for ArgumentFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_binary(f)
    }
}

impl<'v, V: FormatArgument> fmt::LowerExp for ArgumentFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_lower_exp(f)
    }
}

impl<'v, V: FormatArgument> fmt::UpperExp for ArgumentFormatter<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_upper_exp(f)
    }
}

/// A type that associates an argument with a name.
pub trait NamedArguments<V: FormatArgument> {
    /// Returns a reference to the argument associated with the given name, if any.
    fn get(&self, key: &str) -> Option<&V>;
}

impl<K, V> NamedArguments<V> for HashMap<K, V>
where
    K: Borrow<str> + Hash + Eq,
    V: FormatArgument,
{
    fn get(&self, key: &str) -> Option<&V> {
        <HashMap<K, V>>::get(self, key)
    }
}

impl<K, V> NamedArguments<V> for HashMap<K, &V>
where
    K: Borrow<str> + Hash + Eq,
    V: FormatArgument,
{
    fn get(&self, key: &str) -> Option<&V> {
        <HashMap<K, &V>>::get(self, key).map(|v| *v)
    }
}

/// A `NamedArguments` implementation that always returns `None`.
pub struct NoNamedArguments;

impl<V> NamedArguments<V> for NoNamedArguments
where
    V: FormatArgument,
{
    fn get(&self, _: &str) -> Option<&V> {
        None
    }
}

/// A type that provides a list of arguments, randomly accessible by their position.
pub trait PositionalArguments<'v, V>
where
    V: 'v + FormatArgument,
{
    /// The type of the iterator that can be used to iterate over arguments.
    type Iter: Iterator<Item = &'v V>;

    /// Returns a reference to the argument at the given index, if any.
    fn get(&self, index: usize) -> Option<&V>;

    /// Creates an iterator over the arguments.
    fn iter(&'v self) -> Self::Iter;
}

impl<'v, V, T> PositionalArguments<'v, V> for T
where
    V: 'v + FormatArgument,
    T: AsRef<[V]> + ?Sized,
{
    type Iter = std::slice::Iter<'v, V>;

    fn get(&self, index: usize) -> Option<&V> {
        <[V]>::get(self.as_ref(), index)
    }

    fn iter(&'v self) -> Self::Iter {
        <[V]>::iter(self.as_ref())
    }
}

/// A 'PositionalArguments` implementation that always returns `None`.
pub struct NoPositionalArguments;

impl<'v, V> PositionalArguments<'v, V> for NoPositionalArguments
where
    V: 'v + FormatArgument,
{
    type Iter = std::iter::Empty<&'v V>;

    fn get(&self, _: usize) -> Option<&V> {
        None
    }

    fn iter(&'v self) -> Self::Iter {
        std::iter::empty()
    }
}

/// A source of values to use when parsing the formatting string.
pub trait ArgumentSource<V>
where
    V: FormatArgument,
{
    /// Returns the next positional argument, if any. Calling `lookup_argument_by_index` does not
    /// affect which value will be returned by the next call to `next_argument`.
    fn next_argument(&mut self) -> Option<&V>;

    /// Returns the positional argument with the given index, if any. Calling
    /// `lookup_argument_by_index` does not affect which value will be returned by the next call to
    /// `next_argument`.
    fn lookup_argument_by_index(&self, idx: usize) -> Option<&V>;

    /// Returns the named argument with the given name, if any.
    fn lookup_argument_by_name(&self, name: &str) -> Option<&V>;
}
