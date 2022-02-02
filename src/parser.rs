//! Provides support for parsing typical Rust formatting strings.
//! 
//! The parser supports all of the features of the formatting strings that are normally passed to
//! the `format!` macro, except for the fill character.

use regex::{Captures, Match};
use std::convert::{TryFrom, TryInto};
use std::fmt;

use crate::argument::{
    ArgumentFormatter, ArgumentSource, FormatArgument, NamedArguments, PositionalArguments
};
use crate::{format_value, Align, Format, Pad, Precision, Repr, Sign, Specifier, Width};

/// A value and its formatting specifier.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Substitution<'v, V: FormatArgument> {
    specifier: Specifier,
    value: &'v V,
    _private: (),
}

impl<'v, V: FormatArgument> Substitution<'v, V> {
    /// Create an `Substitution` if the given value supports the given format.
    pub fn new(specifier: Specifier, value: &'v V) -> Result<Substitution<'v, V>, ()> {
        if value.supports_format(&specifier) {
            Ok(Substitution {
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

impl<'v, V: FormatArgument> fmt::Display for Substitution<'v, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_value(&self.specifier, &ArgumentFormatter(self.value), f)
    }
}

/// A single segment of a formatting string.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Segment<'s, V: FormatArgument> {
    /// Text to be sent to the formatter.
    Text(&'s str),
    /// A value ready to be formatted.
    Substitution(Substitution<'s, V>),
}

impl<'s, V: FormatArgument> fmt::Display for Segment<'s, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Segment::Text(text) => f.write_str(text),
            Segment::Substitution(arg) => arg.fmt(f),
        }
    }
}

/// A representation of the formatting string and associated values, ready to be formatted.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFormat<'a, V: FormatArgument> {
    /// A vector of formatting string segments.
    pub segments: Vec<Segment<'a, V>>,
}

impl<'a, V: FormatArgument + ConvertToSize> ParsedFormat<'a, V> {
    /// Parses the formatting string, using given positional and named arguments. Does not perform
    /// any formatting. It just parses the formatting string, validates that all the arguments are
    /// present, and that each argument supports the requested format.
    pub fn parse<P, N>(format: &'a str, positional: &'a P, named: &'a N) -> Result<Self, usize>
    where
        P: PositionalArguments<'a, V> + ?Sized,
        N: NamedArguments<V>,
    {
        let segments: Result<Vec<Segment<'a, V>>, usize> =
            Parser::new(format, positional, named).collect();
        Ok(ParsedFormat {
            segments: segments?,
        })
    }
}

impl<'a, V: FormatArgument> fmt::Display for ParsedFormat<'a, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for segment in &self.segments {
            segment.fmt(f)?
        }
        Ok(())
    }
}

/// A type conversion into `usize` that might fail. Like `TryInto<usize>`, but does not consume
/// `self`. The parser needs this trait to support formats whose width or precision use "dollar
/// syntax". For more information about these, see [std::fmt].
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

/// A specifier component that can be parsed from the corresponding part of the formatting string.
trait Parseable<'m, V, S>
where
    V: FormatArgument + ConvertToSize,
    S: ArgumentSource<V>,
    Self: Sized,
{
    fn parse(capture: Option<Match<'m>>, value_src: &mut S) -> Result<Self, ()>;
}

impl<'m, V, S, T> Parseable<'m, V, S> for T
where
    V: FormatArgument + ConvertToSize,
    S: ArgumentSource<V>,
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
    V: FormatArgument + ConvertToSize,
    S: ArgumentSource<V>,
{
    if text.ends_with('$') {
        let text = &text[..text.len() - 1];
        let value = if text.as_bytes()[0].is_ascii_digit() {
            text.parse()
                .ok()
                .and_then(|idx| value_src.lookup_argument_by_index(idx))
        } else {
            value_src.lookup_argument_by_name(text)
        };
        value.ok_or(()).and_then(ConvertToSize::convert)
    } else {
        text.parse().map_err(|_| ())
    }
}

impl<'m, V, S> Parseable<'m, V, S> for Width
where
    V: FormatArgument + ConvertToSize,
    S: ArgumentSource<V>,
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
    V: FormatArgument + ConvertToSize,
    S: ArgumentSource<V>,
{
    fn parse(capture: Option<Match<'m>>, value_src: &mut S) -> Result<Self, ()> {
        match capture.map(|m| m.as_str()).unwrap_or("") {
            "" => Ok(Precision::Auto),
            "*" => value_src
                .next_argument()
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
    V: FormatArgument + ConvertToSize,
    S: ArgumentSource<V>,
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
/// argument specification `{foo:#X}`, this function would parse only the `#X` part.
pub fn parse_specifier<V, S>(spec_str: &str, value_src: &mut S) -> Result<Specifier, ()>
where
    V: FormatArgument + ConvertToSize,
    S: ArgumentSource<V>,
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
pub struct Parser<'p, V, P, N>
where
    V: FormatArgument + ConvertToSize,
    P: PositionalArguments<'p, V> + ?Sized,
    N: NamedArguments<V>,
{
    unparsed: &'p str,
    parsed_len: usize,
    positional: &'p P,
    named: &'p N,
    positional_iter: P::Iter,
}

impl<'p, V, P, N> Parser<'p, V, P, N>
where
    V: FormatArgument + ConvertToSize,
    P: PositionalArguments<'p, V> + ?Sized,
    N: NamedArguments<V>,
{
    /// Creates a new `Parser` for the given formatting string, positional arguments, and named
    /// arguments.
    pub fn new(format: &'p str, positional: &'p P, named: &'p N) -> Self {
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
            self.parse_substitution()
        }
    }

    fn parse_substitution(&mut self) -> Result<Segment<'p, V>, usize> {
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
                    .lookup_argument(&captures)
                    .ok_or(())
                    .and_then(|value| Substitution::new(specifier, value))
                    .map(|arg| {
                        self.advance_and_return(
                            captures.get(0).unwrap().end(),
                            Segment::Substitution(arg),
                        )
                    })
                    .or_else(|_| self.error()),
                Err(_) => self.error(),
            },
        }
    }

    fn next_argument(&mut self) -> Option<&'p V> {
        self.positional_iter.next()
    }

    fn lookup_argument_by_index(&self, idx: usize) -> Option<&'p V> {
        self.positional.get(idx)
    }

    fn lookup_argument_by_name(&self, name: &str) -> Option<&'p V> {
        self.named.get(name)
    }

    fn lookup_argument(&mut self, captures: &Captures) -> Option<&'p V> {
        if let Some(idx) = captures.name("index") {
            idx.as_str()
                .parse::<usize>()
                .ok()
                .and_then(|idx| self.lookup_argument_by_index(idx))
        } else if let Some(name) = captures.name("name") {
            self.lookup_argument_by_name(name.as_str())
        } else {
            self.next_argument()
        }
    }
}

impl<'p, V, P, N> ArgumentSource<V> for Parser<'p, V, P, N>
where
    V: FormatArgument + ConvertToSize,
    P: PositionalArguments<'p, V> + ?Sized,
    N: NamedArguments<V>,
{
    fn next_argument(&mut self) -> Option<&V> {
        (self as &mut Parser<'p, V, P, N>).next_argument()
    }

    fn lookup_argument_by_index(&self, idx: usize) -> Option<&V> {
        (self as &Parser<'p, V, P, N>).lookup_argument_by_index(idx)
    }

    fn lookup_argument_by_name(&self, name: &str) -> Option<&V> {
        (self as &Parser<'p, V, P, N>).lookup_argument_by_name(name)
    }
}

impl<'p, V, P, N> Iterator for Parser<'p, V, P, N>
where
    V: FormatArgument + ConvertToSize,
    P: PositionalArguments<'p, V> + ?Sized,
    N: NamedArguments<V>,
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
