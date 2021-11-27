//! Provides support for parsing typical Rust formatting strings.
//! 
//! The parser supports all of the features of the formatting strings that are normally passed to
//! the `format!` macro, except for the fill character.

use regex::{Captures, Match};
use std::convert::{TryFrom, TryInto};
use std::fmt::Formatter;
use std::slice::Iter;

use crate::argument::{Argument, Segment};
use crate::map::Map;
use crate::value::FormattableValue;
use crate::{Align, Format, Pad, Precision, Repr, Sign, Specifier, Width};

/// A type conversion into `usize` that might fail, similar to `TryInto`. Does not consume `self`.
pub trait ConvertToSize<'s> {
    /// Tries perform the conversion.
    fn convert(&'s self) -> Result<usize, ()>;
}

impl<'t, T: 't> ConvertToSize<'t> for T
where
    &'t T: TryInto<usize, Error = ()>,
{
    fn convert(&'t self) -> Result<usize, ()> {
        self.try_into()
    }
}

/// An iterator of `Segment`s that correspond to the parts of the formatting string being parsed.
pub struct Parser<'p, V, M>
where
    V: FormattableValue + ConvertToSize<'p>,
    M: Map<str, V>,
{
    unparsed: &'p str,
    parsed_len: usize,
    positional: &'p [V],
    named: &'p M,
    positional_iter: Iter<'p, V>,
}

/// A specifier component that can be parsed from the corresponding part of the formatting string.
trait Parseable<'p, 'm, V, M>
where
    V: FormattableValue + ConvertToSize<'p>,
    M: Map<str, V>,
    Self: Sized,
{
    /// Parse the specifier component from the given regex capture group.
    ///
    /// It's also passed the parser to fetch specifier-arguments from (e.g. `5.precision$`).
    /// Returns an error if there is an error in the capture group or if the parser isn't passed
    /// but specifier-arguments are used.
    fn parse(capture: Option<Match<'m>>, parser: Option<&mut Parser<'p, V, M>>) -> Result<Self, ()>;
}

impl<'p, 'm, V, M, T> Parseable<'p, 'm, V, M> for T
where
    V: FormattableValue + ConvertToSize<'p>,
    M: Map<str, V>,
    T: Sized + TryFrom<&'m str, Error = ()>,
{
    fn parse(capture: Option<Match<'m>>, _: Option<&mut Parser<'p, V, M>>) -> Result<Self, ()> {
        capture.map(|m| m.as_str()).unwrap_or("").try_into()
    }
}

/// Parses a size specifier, such as width or precision. If the size is not hard-coded in the
/// formatting string, looks up the corresponding argument and tries to convert it to `usize`.
fn parse_size<'p, 'm, V, M>(text: &str, parser: Option<&mut Parser<'p, V, M>>) -> Result<usize, ()>
where
    V: FormattableValue + ConvertToSize<'p>,
    M: Map<str, V>,
{
    if text.ends_with('$') && parser.is_some() {
        let parser = parser.unwrap();
        let text = &text[..text.len() - 1];
        let value = if text.as_bytes()[0].is_ascii_digit() {
            text.parse()
                .ok()
                .and_then(|idx| parser.lookup_value_by_index(idx))
        } else {
            parser.lookup_value_by_name(text)
        };
        value.ok_or(()).and_then(ConvertToSize::convert)
    } else {
        text.parse().map_err(|_| ())
    }
}

impl<'p, 'm, V, M> Parseable<'p, 'm, V, M> for Width
where
    V: FormattableValue + ConvertToSize<'p>,
    M: Map<str, V>,
{
    fn parse(capture: Option<Match<'m>>, parser: Option<&mut Parser<'p, V, M>>) -> Result<Self, ()> {
        match capture.map(|m| m.as_str()).unwrap_or("") {
            "" => Ok(Width::Auto),
            s @ _ => parse_size(s, parser).map(|width| Width::AtLeast { width }),
        }
    }
}

impl<'p, 'm, V, M> Parseable<'p, 'm, V, M> for Precision
where
    V: FormattableValue + ConvertToSize<'p>,
    M: Map<str, V>,
{
    fn parse(capture: Option<Match<'m>>, mut parser: Option<&mut Parser<'p, V, M>>) -> Result<Self, ()> {
        match (capture.map(|m| m.as_str()).unwrap_or(""), parser.as_deref_mut()) {
            ("", _) => Ok(Precision::Auto),
            ("*", Some(parser)) => parser
                .next_value()
                .ok_or(())
                .and_then(ConvertToSize::convert)
                .map(|precision| Precision::Exactly { precision }),
            (s, _) => parse_size(s, parser).map(|precision| Precision::Exactly { precision }),
        }
    }
}

macro_rules! spec_inner_re {
    () => (r"
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
    ");
}

impl<'p, V, M> Parser<'p, V, M>
where
    V: FormattableValue + ConvertToSize<'p>,
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
            static ref SPEC_RE: Regex = Regex::new(
                concat!(
                    r"(?x)
                        ^
                        \{
                            (?:(?P<index>\d+)|(?P<name>[[:alpha:]][[:alnum:]]*))?
                            (?:
                                :
                    ",
                    spec_inner_re!(),
                    r"
                            )?
                        \}
                    ",
                )
            )
            .unwrap();
        }

        match SPEC_RE.captures(self.unparsed) {
            None => self.error(),
            Some(captures) => match parse_specifier(&captures, Some(self)) {
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

    fn next_value(&mut self) -> Option<&'p V> {
        self.positional_iter.next()
    }

    fn lookup_value_by_index(&self, idx: usize) -> Option<&'p V> {
        self.positional.get(idx)
    }

    fn lookup_value_by_name(&self, name: &str) -> Option<&'p V> {
        self.named.get(name)
    }
}

pub(crate) fn parse_specifier_from_str(spec_str: &str) -> Result<Specifier, ()> {
    use lazy_static::lazy_static;
    use regex::Regex;
    use std::collections::HashMap;

    struct DummyValue;
    impl FormattableValue for DummyValue {
        fn supports_format(&self, _: &Specifier) -> bool { unreachable!() }
        fn fmt_display(&self, _: &mut Formatter) -> std::fmt::Result { unreachable!() }
        fn fmt_debug(&self, _: &mut Formatter) -> std::fmt::Result { unreachable!() }
        fn fmt_octal(&self, _: &mut Formatter) -> std::fmt::Result { unreachable!() }
        fn fmt_lower_hex(&self, _: &mut Formatter) -> std::fmt::Result { unreachable!() }
        fn fmt_upper_hex(&self, _: &mut Formatter) -> std::fmt::Result { unreachable!() }
        fn fmt_binary(&self, _: &mut Formatter) -> std::fmt::Result { unreachable!() }
        fn fmt_lower_exp(&self, _: &mut Formatter) -> std::fmt::Result { unreachable!() }
        fn fmt_upper_exp(&self, _: &mut Formatter) -> std::fmt::Result { unreachable!() }
    }
    impl<'p> ConvertToSize<'p> for DummyValue {
        fn convert(&'p self) -> Result<usize, ()> { unreachable!() }
    }

    lazy_static! {
            static ref SPEC_INNER_RE: Regex = Regex::new(
                concat!(
                    r"(?x)
                        ^
                    ",
                    spec_inner_re!(),
                )
            )
            .unwrap();
    }
    match SPEC_INNER_RE.captures(spec_str) {
        None => Err(()),
        Some(captures) => parse_specifier::<DummyValue, HashMap<&str, DummyValue>>(&captures, None),
    }
}

fn parse_specifier<'p, V, M>(captures: &Captures, mut parser: Option<&mut Parser<'p, V, M>>) -> Result<Specifier, ()>
where
    V: FormattableValue + ConvertToSize<'p>,
    M: Map<str, V>,
{
    Ok(Specifier {
        align: Align::parse(captures.name("align"), parser.as_deref_mut())?,
        sign: Sign::parse(captures.name("sign"), parser.as_deref_mut())?,
        repr: Repr::parse(captures.name("repr"), parser.as_deref_mut())?,
        pad: Pad::parse(captures.name("pad"), parser.as_deref_mut())?,
        width: Width::parse(captures.name("width"), parser.as_deref_mut())?,
        precision: Precision::parse(captures.name("precision"), parser.as_deref_mut())?,
        format: Format::parse(captures.name("format"), parser)?,
    })
}


impl<'p, V, M> Iterator for Parser<'p, V, M>
where
    V: FormattableValue + ConvertToSize<'p>,
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
