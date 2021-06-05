use regex::{Captures, Match};
use std::convert::{TryFrom, TryInto};
use std::slice::Iter;

use crate::{Align, Argument, Format, FormattableValue, Pad, Precision, Repr, Segment, Sign, Specifier, Width};
use crate::map::Map;

pub struct Parser<'p, V, M>
where
    V: FormattableValue,
    M: Map<str, V>
{
    unparsed: &'p str,
    parsed_len: usize,
    positional: &'p [V],
    named: &'p M,
    positional_iter: Iter<'p, V>,
}

trait Parseable<'p, 'm, V, M>
where
    V: FormattableValue,
    M: Map<str, V>,
    Self: Sized
{
    fn parse(capture: Option<Match<'m>>, parser: &mut Parser<'p, V, M>) -> Result<Self, ()>;
}

impl<'p, 'm, V, M, T> Parseable<'p, 'm, V, M> for T
where
    V: FormattableValue,
    M: Map<str, V>,
    T: Sized + TryFrom<&'m str, Error = ()>
{
    fn parse(capture: Option<Match<'m>>, _: &mut Parser<'p, V, M>) -> Result<Self, ()> {
        capture.map(|m| m.as_str()).unwrap_or("").try_into()
    }
}

impl<'p, 'm, V, M> Parseable<'p, 'm, V, M> for Width
where
    V: FormattableValue,
    M: Map<str, V>
{
    fn parse(capture: Option<Match<'m>>, _: &mut Parser<'p, V, M>) -> Result<Self, ()> {
        match capture.map(|m| m.as_str()).unwrap_or("") {
            "" => Ok(Width::Auto),
            s@_ => match s.parse() {
                Ok(width) => Ok(Width::AtLeast { width }),
                Err(_) => Err(())
            }
        }
    }
}

impl<'p, 'm, V, M> Parseable<'p, 'm, V, M> for Precision
where
    V: FormattableValue,
    M: Map<str, V>
{
    fn parse(capture: Option<Match<'m>>, _: &mut Parser<'p, V, M>) -> Result<Self, ()> {
        match capture.map(|m| m.as_str()).unwrap_or("") {
            "" => Ok(Precision::Auto),
            s@_ => match s.parse() {
                Ok(precision) => Ok(Precision::Exactly { precision }),
                Err(_) => Err(())
            }
        }
    }
}

impl<'p, V, M> Parser<'p, V, M>
where
    V: FormattableValue,
    M: Map<str, V>
{
    pub fn new(format: &'p str, positional: &'p [V], named: &'p M) -> Self {
        Parser { unparsed: format, parsed_len: 0, positional, named, positional_iter: positional.iter() }
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

    fn parse_specifier(&mut self, captures: &Captures) -> Specifier {
        Specifier {
            align: Align::parse(captures.name("align"), self).unwrap(),
            sign: Sign::parse(captures.name("sign"), self).unwrap(),
            repr: Repr::parse(captures.name("repr"), self).unwrap(),
            pad: Pad::parse(captures.name("pad"), self).unwrap(),
            width: Width::parse(captures.name("width"), self).unwrap(),
            precision: Precision::parse(captures.name("precision"), self).unwrap(),
            format: Format::parse(captures.name("format"), self).unwrap(),
        }
    }

    fn parse_argument(&mut self) -> Result<Segment<'p, V>, usize> {
        use regex::Regex;
        use lazy_static::lazy_static;
    
        lazy_static! {
            static ref SPEC_RE: Regex = Regex::new(r"(?x)
                ^
                \{
                    (?:(?P<index>\d+)|(?P<name>[[:alpha:]][[:alnum:]]*))?
                    (?:
                        :
                        (?P<align>[<^>])?
                        (?P<sign>\+)?
                        (?P<repr>\#)?
                        (?P<pad>0)?
                        (?P<width>\d+)?
                        (?:\.(?P<precision>\d+))?
                        (?P<format>[?oxXbeE])?
                    )?
                \}
            ").unwrap();
        }

        match SPEC_RE.captures(self.unparsed) {
            None => self.error(),
            Some(captures) => {
                self.lookup_value(&captures)
                    .ok_or(())
                    .and_then(|value| Argument::new(self.parse_specifier(&captures), value))
                    .map(|arg| self.advance_and_return(captures.get(0).unwrap().end(), Segment::Argument(arg)))
                    .or_else(|_| self.error())
            }
        }
    }

    fn lookup_value(&mut self, captures: &Captures) -> Option<&'p V> {
        if let Some(idx) = captures.name("index") {
            idx.as_str().parse::<usize>().ok().and_then(|idx| self.positional.get(idx))
        } else if let Some(name) = captures.name("name") {
            self.named.get(name.as_str())
        } else {
            self.positional_iter.next()
        }
    }
}

impl<'p, V, M> Iterator for Parser<'p, V, M>
where
    V: FormattableValue,
    M: Map<str, V>
{
    type Item = Result<Segment<'p, V>, usize>;

    fn next(&mut self) -> Option<Self::Item> {
        static BRACES: &[char] = &['{', '}'];

        if self.unparsed.len() == 0 {
            return None
        }
        
        match self.unparsed.find(BRACES) {
            None => Some(Ok(self.text_segment(self.unparsed.len()))),
            Some(0) => Some(self.parse_braces()),
            Some(brace_idx) => Some(Ok(self.text_segment(brace_idx))),
        }
    }
}
