//! Provides support for parsing typical Rust formatting strings.
//! 
//! The parser supports all of the features of the formatting strings that are normally passed to
//! the `format!` macro, except for the fill character.

use regex::{Captures, Match};
use std::convert::{TryFrom, TryInto};
use std::slice::Iter;

use crate::argument::{Argument, Segment};
use crate::map::Map;
use crate::value::FormattableValue;
use crate::{Align, Format, Pad, Precision, Repr, Sign, Specifier, Width};

/// A type conversion into `usize` that might fail, similar to `TryInto`. Does not consume `self`.
pub trait ConvertToSize {
    /// Tries perform the conversion.
    fn convert(&self) -> Result<usize, ()>;
}

impl<T> ConvertToSize for T
where
    for<'t> &'t T: TryInto<usize, Error = ()>,
{
    fn convert(&self) -> Result<usize, ()> {
        self.try_into()
    }
}

/// A source of values to use when parsing the formatting string.
pub trait ValueSource<V>
where
    V: FormattableValue,
{
    /// Returns the next positional argument, if any. Calling `lookup_value_by_index` does not
    /// affect which value will be returned by the next call to `next_value`.
    fn next_value(&mut self) -> Option<&V>;

    /// Returns the positional argument with the given index, if any. Calling
    /// `lookup_value_by_index` does not affect which value will be returned by the next call to
    /// `next_value`.
    fn lookup_value_by_index(&self, idx: usize) -> Option<&V>;

    /// Returns the named argument with the given name, if any.
    fn lookup_value_by_name(&self, name: &str) -> Option<&V>;
}

/// A specifier component that can be parsed from the corresponding part of the formatting string.
trait Parseable<'m, V, S>
where
    V: FormattableValue + ConvertToSize,
    S: ValueSource<V>,
    Self: Sized,
{
    fn parse(capture: Option<Match<'m>>, value_src: &mut S) -> Result<Self, ()>;
}

impl<'m, V, S, T> Parseable<'m, V, S> for T
where
    V: FormattableValue + ConvertToSize,
    S: ValueSource<V>,
    T: Sized + TryFrom<&'m str, Error = ()>,
{
    fn parse(capture: Option<Match<'m>>, _: &mut S) -> Result<Self, ()> {
        capture.map(|m| m.as_str()).unwrap_or("").try_into()
    }
}

/// Parses a size specifier, such as width or precision. If the size is not hard-coded in the
/// formatting string, looks up the corresponding argument and tries to convert it to `usize`.
fn parse_size<'m, V, S>(text: &str, value_src: &S) -> Result<usize, ()>
where
    V: FormattableValue + ConvertToSize,
    S: ValueSource<V>,
{
    if text.ends_with('$') {
        let text = &text[..text.len() - 1];
        let value = if text.as_bytes()[0].is_ascii_digit() {
            text.parse()
                .ok()
                .and_then(|idx| value_src.lookup_value_by_index(idx))
        } else {
            value_src.lookup_value_by_name(text)
        };
        value.ok_or(()).and_then(ConvertToSize::convert)
    } else {
        text.parse().map_err(|_| ())
    }
}

impl<'m, V, S> Parseable<'m, V, S> for Width
where
    V: FormattableValue + ConvertToSize,
    S: ValueSource<V>,
{
    fn parse(capture: Option<Match<'m>>, value_src: &mut S) -> Result<Self, ()> {
        match capture.map(|m| m.as_str()).unwrap_or("") {
            "" => Ok(Width::Auto),
            s @ _ => parse_size(s, value_src).map(|width| Width::AtLeast { width }),
        }
    }
}

impl<'m, V, S> Parseable<'m, V, S> for Precision
where
    V: FormattableValue + ConvertToSize,
    S: ValueSource<V>,
{
    fn parse(capture: Option<Match<'m>>, value_src: &mut S) -> Result<Self, ()> {
        match capture.map(|m| m.as_str()).unwrap_or("") {
            "" => Ok(Precision::Auto),
            "*" => value_src
                .next_value()
                .ok_or(())
                .and_then(ConvertToSize::convert)
                .map(|precision| Precision::Exactly { precision }),
            s @ _ => parse_size(s, value_src).map(|precision| Precision::Exactly { precision }),
        }
    }
}

macro_rules! SPEC_REGEX_FRAG {
    () => { r"
        (?P<align>[<^>])?
        (?P<sign>\+)?
        (?P<repr>\#)?
        (?P<pad>0)?
        (?P<width>
            (?:\d+\$?)|(?:[[:alpha:]][[:alnum:]]*\$)
        )?
        (?:\.(?P<precision>
            (?:\d+\$?)|(?:[[:alpha:]][[:alnum:]]*\$)|\*
        ))?
        (?P<format>[?oxXbeE])?
    " };
}

fn parse_specifier_captures<V, S>(captures: &Captures, value_src: &mut S) -> Result<Specifier, ()>
where
    V: FormattableValue + ConvertToSize,
    S: ValueSource<V>,
{
    Ok(Specifier {
        align: Align::parse(captures.name("align"), value_src)?,
        sign: Sign::parse(captures.name("sign"), value_src)?,
        repr: Repr::parse(captures.name("repr"), value_src)?,
        pad: Pad::parse(captures.name("pad"), value_src)?,
        width: Width::parse(captures.name("width"), value_src)?,
        precision: Precision::parse(captures.name("precision"), value_src)?,
        format: Format::parse(captures.name("format"), value_src)?,
    })
}

/// Parses only the format specifier portion of a format argument. For example, in a format
/// argument specification "{foo:#X}", this function would parse only the "#X" part.
pub fn parse_specifier<V, S>(spec_str: &str, value_src: &mut S) -> Result<Specifier, ()>
where
    V: FormattableValue + ConvertToSize,
    S: ValueSource<V>,
{
    use lazy_static::lazy_static;
    use regex::Regex;

    lazy_static! {
        static ref SPEC_RE: Regex = Regex::new(concat!(r"(?x) ^", SPEC_REGEX_FRAG!())).unwrap();
    }

    match SPEC_RE.captures(spec_str) {
        None => Err(()),
        Some(captures) => parse_specifier_captures(&captures, value_src)
    }
}

/// An iterator of `Segment`s that correspond to the parts of the formatting string being parsed.
pub struct Parser<'p, V, M>
where
    V: FormattableValue + ConvertToSize,
    M: Map<str, V>,
{
    unparsed: &'p str,
    parsed_len: usize,
    positional: &'p [V],
    named: &'p M,
    positional_iter: Iter<'p, V>,
}

impl<'p, V, M> Parser<'p, V, M>
where
    V: FormattableValue + ConvertToSize,
    M: Map<str, V>,
{
    /// Creates a new `Parser` for the given formatting string, positional arguments, and named
    /// arguments.
    pub fn new(format: &'p str, positional: &'p [V], named: &'p M) -> Self {
        Parser {
            unparsed: format,
            parsed_len: 0,
            positional,
            named,
            positional_iter: positional.iter(),
        }
    }

    fn advance_and_return<T>(&mut self, advance_by: usize, result: T) -> T {
        self.unparsed = &self.unparsed[advance_by..];
        self.parsed_len += advance_by;
        result
    }

    fn error(&mut self) -> Result<Segment<'p, V>, usize> {
        self.unparsed = "";
        Err(self.parsed_len)
    }

    fn text_segment(&mut self, len: usize) -> Segment<'p, V> {
        self.advance_and_return(len, Segment::Text(&self.unparsed[..len]))
    }

    fn parse_braces(&mut self) -> Result<Segment<'p, V>, usize> {
        if self.unparsed.len() < 2 {
            self.error()
        } else if self.unparsed.as_bytes()[0] == self.unparsed.as_bytes()[1] {
            Ok(self.advance_and_return(2, Segment::Text(&self.unparsed[..1])))
        } else {
            self.parse_argument()
        }
    }

    fn parse_argument(&mut self) -> Result<Segment<'p, V>, usize> {
        use lazy_static::lazy_static;
        use regex::Regex;

        lazy_static! {
            static ref ARG_RE: Regex = Regex::new(
                concat!(
                    r"(?x)
                        ^
                        \{
                            (?:(?P<index>\d+)|(?P<name>[[:alpha:]][[:alnum:]]*))?
                            (?:
                                :
                    ",
                    SPEC_REGEX_FRAG!(),
                    r"
                            )?
                    \}"
                )
            )
            .unwrap();
        }

        match ARG_RE.captures(self.unparsed) {
            None => self.error(),
            Some(captures) => match parse_specifier_captures(&captures, self) {
                Ok(specifier) => self
                    .lookup_value(&captures)
                    .ok_or(())
                    .and_then(|value| Argument::new(specifier, value))
                    .map(|arg| {
                        self.advance_and_return(
                            captures.get(0).unwrap().end(),
                            Segment::Argument(arg),
                        )
                    })
                    .or_else(|_| self.error()),
                Err(_) => self.error(),
            },
        }
    }

    fn next_value(&mut self) -> Option<&'p V> {
        self.positional_iter.next()
    }

    fn lookup_value_by_index(&self, idx: usize) -> Option<&'p V> {
        self.positional.get(idx)
    }

    fn lookup_value_by_name(&self, name: &str) -> Option<&'p V> {
        self.named.get(name)
    }

    fn lookup_value(&mut self, captures: &Captures) -> Option<&'p V> {
        if let Some(idx) = captures.name("index") {
            idx.as_str()
                .parse::<usize>()
                .ok()
                .and_then(|idx| self.lookup_value_by_index(idx))
        } else if let Some(name) = captures.name("name") {
            self.lookup_value_by_name(name.as_str())
        } else {
            self.next_value()
        }
    }
}

impl<'p, V, M> ValueSource<V> for Parser<'p, V, M>
where
    V: FormattableValue + ConvertToSize,
    M: Map<str, V>,
{
    fn next_value(&mut self) -> Option<&V> {
        (self as &mut Parser<'p, V, M>).next_value()
    }

    fn lookup_value_by_index(&self, idx: usize) -> Option<&V> {
        (self as &Parser<'p, V, M>).lookup_value_by_index(idx)
    }

    fn lookup_value_by_name(&self, name: &str) -> Option<&V> {
        (self as &Parser<'p, V, M>).lookup_value_by_name(name)
    }
}

impl<'p, V, M> Iterator for Parser<'p, V, M>
where
    V: FormattableValue + ConvertToSize,
    M: Map<str, V>,
{
    type Item = Result<Segment<'p, V>, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        static BRACES: &[char] = &['{', '}'];

        if self.unparsed.len() == 0 {
            return None;
        }

        match self.unparsed.find(BRACES) {
            None => Some(Ok(self.text_segment(self.unparsed.len()))),
            Some(0) => Some(self.parse_braces()),
            Some(brace_idx) => Some(Ok(self.text_segment(brace_idx))),
        }
    }
}
