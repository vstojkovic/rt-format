use std::collections::HashMap;

use rt_format::argument::{
    ArgumentSource, NamedArguments, NoNamedArguments, NoPositionalArguments, PositionalArguments
};
use rt_format::parser::{parse_specifier};
use rt_format::{Align, ParsedFormat, Format, Pad, Precision, Repr, Sign, Specifier, Width};

mod common;
use common::Variant;

type ParseResult<'a> = Result<ParsedFormat<'a, Variant>, usize> ;

fn parse<'a, P, N>(format: &'a str, positional: &'a P, named: &'a N) -> ParseResult<'a>
where
    P: PositionalArguments<'a, Variant>,
    N: NamedArguments<Variant>,
{
    ParsedFormat::parse(format, positional, named)
}

#[test]
fn unmatched_brace() {
    assert_eq!(Err(4), parse("foo {", &NoPositionalArguments, &NoNamedArguments));
    assert_eq!(Err(4), parse("bar } baz", &NoPositionalArguments, &NoNamedArguments));
}

#[test]
fn escaped_braces() {
    assert_eq!(
        "{}",
        parse("{{}}", &NoPositionalArguments, &NoNamedArguments)
            .unwrap()
            .to_string()
    );
}

#[test]
fn invalid_specifier() {
    assert_eq!(
        Err(4),
        parse("foo {:Z} bar", &[Variant::Int(42)], &NoNamedArguments)
    );
}

#[test]
fn invalid_arg_position() {
    assert_eq!(
        Err(4),
        parse("foo {0bar} baz", &[Variant::Int(42)], &NoNamedArguments)
    );
}

#[test]
fn positional_arg_iter() {
    assert_eq!(
        "42 42.042",
        parse("{} {}", &[Variant::Int(42), Variant::Float(42.042)], &NoNamedArguments)
            .unwrap()
            .to_string()
    );
}

#[test]
fn positional_arg_lookup() {
    assert_eq!(
        "42.042",
        parse("{1}", &[Variant::Int(42), Variant::Float(42.042)], &NoNamedArguments)
            .unwrap()
            .to_string()
    );
}

#[test]
fn named_arg_lookup() {
    let mut map = HashMap::new();
    map.insert("arglebargle".to_string(), Variant::Float(-42.042));
    assert_eq!(
        "-42.042",
        parse("{arglebargle}", &NoPositionalArguments, &map)
            .unwrap()
            .to_string()
    );
}

#[test]
fn missing_next_arg() {
    assert_eq!(
        Err(3),
        parse("{} {}", &[Variant::Int(42)], &NoNamedArguments)
    );
}

#[test]
fn missing_positional_arg() {
    assert_eq!(Err(0), parse("{1}", &[Variant::Int(42)], &NoNamedArguments));
}

#[test]
fn missing_named_arg() {
    assert_eq!(Err(0), parse("{arglebargle}", &NoPositionalArguments, &NoNamedArguments));
}

#[test]
fn missing_positional_width() {
    assert_eq!(
        Err(0),
        parse("{:1$}", &[Variant::Int(42)], &NoNamedArguments)
    );
}

#[test]
fn missing_named_width() {
    assert_eq!(
        Err(0),
        parse("{:arglebargle$}", &[Variant::Int(42)], &NoNamedArguments)
    );
}

#[test]
fn missing_positional_precision() {
    assert_eq!(
        Err(0),
        parse("{:.1$}", &[Variant::Int(42)], &NoNamedArguments)
    );
}

#[test]
fn missing_named_precision() {
    assert_eq!(
        Err(0),
        parse("{:.arglebargle$}", &[Variant::Int(42)], &NoNamedArguments)
    );
}

#[test]
fn missing_asterisk_precision() {
    assert_eq!(
        Err(3),
        parse("{} {0:.*}", &[Variant::Int(42)], &NoNamedArguments)
    );
}

#[test]
fn named_argument_validity() {
    let mut map = HashMap::new();
    map.insert("ascii_identifier".to_string(), Variant::Int(42));
    map.insert("_leading_underscore".to_string(), Variant::Int(4242));
    map.insert("уникод".to_string(), Variant::Float(42.042));
    map.insert("0leading_digit".to_string(), Variant::Int(-42));
    map.insert("invalid/character".to_string(), Variant::Float(-42.042));

    assert!(parse("{ascii_identifier}", &NoPositionalArguments, &map).is_ok());
    assert!(parse("{_leading_underscore}", &NoPositionalArguments, &map).is_ok());
    assert!(parse("{уникод}", &NoPositionalArguments, &map).is_ok());

    assert_eq!(Err(0), parse("{0leading_digit}", &NoPositionalArguments, &map));
    assert_eq!(Err(0), parse("{invalid/character}", &NoPositionalArguments, &map));
}

#[test]
fn parse_specifier_smoke_test() {
    struct NoValues;
    impl ArgumentSource<Variant> for NoValues {
        fn next_argument(&mut self) -> Option<&Variant> { None }
        fn lookup_argument_by_index(&self, _: usize) -> Option<&Variant> { None }
        fn lookup_argument_by_name(&self, _: &str) -> Option<&Variant> { None }
    }

    assert_eq!(
        Ok(Specifier {
            align: Align::Right,
            sign: Sign::Always,
            repr: Repr::Alt,
            pad: Pad::Zero,
            width: Width::AtLeast { width: 42 },
            precision: Precision::Exactly { precision: 17 },
            format: Format::UpperExp,
        }),
        parse_specifier(">+#042.17E", &mut NoValues {})
    );
}
