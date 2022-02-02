use rt_format::{Align, Format, Pad, Precision, Repr, Sign, Specifier, Width};

#[test]
fn specifier_traits() {
    assert_eq!("+#o", format!("{}", Specifier {
        sign: Sign::Always,
        repr: Repr::Alt,
        format: Format::Octal,
        ..Default::default()
    }));
    assert_eq!("^042.17E", format!("{}", Specifier {
        align: Align::Center,
        pad: Pad::Zero,
        width: Width::AtLeast { width: 42 },
        precision: Precision::Exactly { precision: 17 },
        format: Format::UpperExp,
        ..Default::default()
    }));
}
