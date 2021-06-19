use std::fmt;

use crate::map::Map;
use crate::parser::ConvertToSize;
use crate::value::{FormattableValue, ValueFormatter};
use crate::{format_value, Specifier};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Argument<'v, V: FormattableValue> {
    pub specifier: Specifier,
    pub value: &'v V,
    _private: (),
}

impl<'v, V: FormattableValue> Argument<'v, V> {
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
}

impl<'v, V: FormattableValue> fmt::Display for Argument<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_value(&self.specifier, &ValueFormatter(self.value), f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Segment<'s, V: FormattableValue> {
    Text(&'s str),
    Argument(Argument<'s, V>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arguments<'a, V: FormattableValue> {
    pub segments: Vec<Segment<'a, V>>,
}

impl<'a, V: FormattableValue + ConvertToSize<'a>> Arguments<'a, V> {
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
            match segment {
                Segment::Text(text) => f.write_str(text)?,
                Segment::Argument(arg) => arg.fmt(f)?,
            }
        }
        Ok(())
    }
}
