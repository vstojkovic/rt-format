use std::collections::HashMap;

use rt_format::{Arguments};
use rt_format::map::NoMap;

mod common;
use common::Variant;

fn fmt_args(spec: &str, args: &[Variant]) -> String {
    format!("{}", Arguments::parse(spec, args, &NoMap).unwrap())
}

#[test]
fn align_left() {
    assert_eq!("#42    #", fmt_args("#{:<6}#", &[Variant::Int(42)]));
}

#[test]
fn align_center() {
    assert_eq!("#  42  #", fmt_args("#{:^6}#", &[Variant::Int(42)]));
}

#[test]
fn align_right() {
    assert_eq!("#    42#", fmt_args("#{:>6}#", &[Variant::Int(42)]));
}

#[test]
fn sign_always() {
    assert_eq!("+42", fmt_args("{:+}", &[Variant::Int(42)]));
}

#[test]
fn reprt_alt_octal() {
    assert_eq!("0o52", fmt_args("{:#o}", &[Variant::Int(42)]));
}

#[test]
fn reprt_alt_lower_hex() {
    assert_eq!("0x2a", fmt_args("{:#x}", &[Variant::Int(42)]));
}

#[test]
fn reprt_alt_upper_hex() {
    assert_eq!("0x2A", fmt_args("{:#X}", &[Variant::Int(42)]));
}

#[test]
fn reprt_alt_binary() {
    assert_eq!("0b101010", fmt_args("{:#b}", &[Variant::Int(42)]));
}

#[test]
fn pad_zero() {
    assert_eq!("#00042#", fmt_args("#{:05}#", &[Variant::Int(42)]));
}

#[test]
fn precision() {
    assert_eq!("#42.04200#", fmt_args("#{:.5}#", &[Variant::Float(42.042)]));
}

#[test]
fn format_display() {
    assert_eq!("42", fmt_args("{}", &[Variant::Int(42)]));
}

#[test]
fn format_octal() {
    assert_eq!("52", fmt_args("{:o}", &[Variant::Int(42)]));
}

#[test]
fn format_lower_hex() {
    assert_eq!("2a", fmt_args("{:x}", &[Variant::Int(42)]));
}

#[test]
fn format_upper_hex() {
    assert_eq!("2A", fmt_args("{:X}", &[Variant::Int(42)]));
}

#[test]
fn format_binary() {
    assert_eq!("101010", fmt_args("{:b}", &[Variant::Int(42)]));
}

#[test]
fn format_lower_exp() {
    assert_eq!("4.2e1", fmt_args("{:e}", &[Variant::Int(42)]));
}

#[test]
fn format_upper_exp() {
    assert_eq!("4.2E1", fmt_args("{:E}", &[Variant::Int(42)]));
}

#[test]
fn smoke_test() {
    let mut map = HashMap::new();
    map.insert("argle".to_string(), Variant::Int(-42));
    assert_eq!(
        "foo +21 # -42 # 0x2A 386 {10001} 42 bar",
        format!("{}", Arguments::parse("foo {:+o} #{argle:^5}# {2:#X} {} {{{0:b}}} {:} bar", &[Variant::Int(17), Variant::Int(386), Variant::Int(42)], &map).unwrap())
    );
}
