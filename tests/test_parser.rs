use std::collections::HashMap;

use rt_format::map::NoMap;
use rt_format::Arguments;

mod common;
use common::Variant;

const NO_ARGS: &[Variant] = &[];

#[test]
fn unmatched_brace() {
    assert_eq!(Err(4), Arguments::parse("foo {", NO_ARGS, &NoMap));
    assert_eq!(Err(4), Arguments::parse("bar } baz", NO_ARGS, &NoMap));
}

#[test]
fn escaped_braces() {
    assert_eq!(
        "{}",
        Arguments::parse("{{}}", NO_ARGS, &NoMap)
            .unwrap()
            .to_string()
    );
}

#[test]
fn invalid_specifier() {
    assert_eq!(
        Err(4),
        Arguments::parse("foo {:Z} bar", &[Variant::Int(42)], &NoMap)
    );
}

#[test]
fn invalid_arg_position() {
    assert_eq!(
        Err(4),
        Arguments::parse("foo {0bar} baz", &[Variant::Int(42)], &NoMap)
    );
}

#[test]
fn positional_arg_iter() {
    assert_eq!(
        "42 42.042",
        Arguments::parse("{} {}", &[Variant::Int(42), Variant::Float(42.042)], &NoMap)
            .unwrap()
            .to_string()
    );
}

#[test]
fn positional_arg_lookup() {
    assert_eq!(
        "42.042",
        Arguments::parse("{1}", &[Variant::Int(42), Variant::Float(42.042)], &NoMap)
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
        Arguments::parse("{arglebargle}", NO_ARGS, &map)
            .unwrap()
            .to_string()
    );
}

#[test]
fn missing_next_arg() {
    assert_eq!(
        Err(3),
        Arguments::parse("{} {}", &[Variant::Int(42)], &NoMap)
    );
}

#[test]
fn missing_positional_arg() {
    assert_eq!(Err(0), Arguments::parse("{1}", &[Variant::Int(42)], &NoMap));
}

#[test]
fn missing_named_arg() {
    assert_eq!(Err(0), Arguments::parse("{arglebargle}", NO_ARGS, &NoMap));
}

#[test]
fn missing_positional_width() {
    assert_eq!(
        Err(0),
        Arguments::parse("{:1$}", &[Variant::Int(42)], &NoMap)
    );
}

#[test]
fn missing_named_width() {
    assert_eq!(
        Err(0),
        Arguments::parse("{:arglebargle$}", &[Variant::Int(42)], &NoMap)
    );
}

#[test]
fn missing_positional_precision() {
    assert_eq!(
        Err(0),
        Arguments::parse("{:.1$}", &[Variant::Int(42)], &NoMap)
    );
}

#[test]
fn missing_named_precision() {
    assert_eq!(
        Err(0),
        Arguments::parse("{:.arglebargle$}", &[Variant::Int(42)], &NoMap)
    );
}

#[test]
fn missing_asterisk_precision() {
    assert_eq!(
        Err(3),
        Arguments::parse("{} {0:.*}", &[Variant::Int(42)], &NoMap)
    );
}
#[test]
fn uiae() {
    assert_eq!(
        Err(1000),
        Arguments::parse("{:x?}", &[Variant::Int(42)], &NoMap)
    );
}
