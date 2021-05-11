use std::collections::HashMap;
use std::fmt;

use rt_format::{Arguments, FormattableValue, Specifier};

struct Int(i32);

impl FormattableValue for Int {
    fn supports_format(&self, _: &Specifier) -> bool {
        true
    }

    fn fmt_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }

    fn fmt_debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }

    fn fmt_octal(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }

    fn fmt_lower_hex(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }

    fn fmt_upper_hex(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }

    fn fmt_binary(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }

    fn fmt_lower_exp(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerExp::fmt(&self.0, f)
    }

    fn fmt_upper_exp(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperExp::fmt(&self.0, f)
    }
}

#[test]
fn smoke_test() {
    let mut map = HashMap::new();
    map.insert("argle".to_string(), Int(-42));
    assert_eq!(
        "foo +21 # -42 # 0x2A 386 {10001} 42 bar",
        format!("{}", Arguments::parse("foo {:+o} #{argle:^5}# {2:#X} {} {{{0:b}}} {:} bar", &[Int(17), Int(386), Int(42)], &map).unwrap())
    );
}
