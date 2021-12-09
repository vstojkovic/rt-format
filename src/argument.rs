//! Provides types that hold the values to format and their associated formatting specifications.

use std::fmt;

use crate::map::Map;
use crate::parser::ConvertToSize;
use crate::value::{FormattableValue, ValueFormatter};
use crate::{format_value, Specifier};

/// A value and its formatting specifier.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Argument<'v, V: FormattableValue> {
    specifier: Specifier,
    value: &'v V,
    _private: (),
}

impl<'v, V: FormattableValue> Argument<'v, V> {
    /// Create an `Argument` if the given value supports the given format.
    pub fn new(specifier: Specifier, value: &'v V) -> Result<Argument<'v, V>, ()> {
        if value.supports_format(&specifier) {
            Ok(Argument {
                specifier,
                value,
                _private: (),
            })
        } else {
            Err(())
        }
    }

    /// A reference to the formatting specifier.
    pub fn specifier(&self) -> &Specifier {
        &self.specifier
    }

    /// A reference to the value to format.
    pub fn value(&self) -> &'v V {
        self.value
    }
}

impl<'v, V: FormattableValue> fmt::Display for Argument<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_value(&self.specifier, &ValueFormatter(self.value), f)
    }
}

/// A single segment of a formatting string.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Segment<'s, V: FormattableValue> {
    /// Text to be sent to the formatter.
    Text(&'s str),
    /// A value ready to be formatted.
    Argument(Argument<'s, V>),
}

impl<'s, V: FormattableValue> fmt::Display for Segment<'s, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Segment::Text(text) => f.write_str(text),
            Segment::Argument(arg) => arg.fmt(f),
        }
    }
}

/// A representation of the formatting string and associated values, ready to be formatted.
#[derive(Debug, Clone, PartialEq)]
pub struct Arguments<'a, V: FormattableValue> {
    /// A vector of formatting string segments.
    pub segments: Vec<Segment<'a, V>>,
}

impl<'a, V: FormattableValue + ConvertToSize> Arguments<'a, V> {
    /// Parses the formatting string, using given positional and named arguments. Does not perform
    /// any formatting. It just parses the formatting string, validates that all the arguments are
    /// present, and that each argument supports the requested format.
    pub fn parse<M>(format: &'a str, positional: &'a [V], named: &'a M) -> Result<Self, usize>
    where
        M: Map<str, V>,
    {
        use crate::parser::Parser;

        let segments: Result<Vec<Segment<'a, V>>, usize> =
            Parser::new(format, positional, named).collect();
        Ok(Arguments {
            segments: segments?,
        })
    }
}

impl<'a, V: FormattableValue> fmt::Display for Arguments<'a, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for segment in self.segments.iter() {
            segment.fmt(f)?
        }
        Ok(())
    }
}
