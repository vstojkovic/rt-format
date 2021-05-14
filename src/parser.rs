use regex::Captures;
use std::slice::Iter;

use crate::{Argument, FormattableValue, Segment, Specifier};
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
                    .and_then(|value| Argument::new(Specifier::new(&captures), value))
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
