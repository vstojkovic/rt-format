#[macro_use]
mod codegen;

pub mod argument;
pub mod map;
pub mod parser;
pub mod value;

use std::cmp::PartialEq;
use std::convert::TryFrom;
use std::fmt;

pub use crate::argument::{Argument, Arguments};
pub use crate::value::FormattableValue;

generate_code! {
    align: Align {
        None => "",
        Left => "<",
        Center => "^",
        Right => ">",
    }

    sign: Sign {
        Default => "",
        Always => "+",
    }

    repr: Repr {
        Default => "",
        Alt => "#",
    }

    pad: Pad {
        Space => "",
        Zero => "0",
    }

    width: Width {
        Auto => "",
        AtLeast { width: usize } => "width$",
    }

    precision: Precision {
        Auto => "",
        Exactly { precision: usize } => ".precision$",
    }

    format: Format {
        Display => "",
        Debug => "?",
        Octal => "o",
        LowerHex => "x",
        UpperHex => "X",
        Binary => "b",
        LowerExp => "e",
        UpperExp => "E",
    }
}
