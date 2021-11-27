#![warn(missing_docs)]

//! Fully-runtime equivalent of the `format!` macro.
//! 
//! Allows formatting strings like the `format!` macro, with the formatting string and the arguments
//! provided at runtime. This crate supports all the formatting features of the `format!` macro,
//! except for the fill character.
//! 
//! # Examples
//! 
//! ```
//! use rt_format::{Arguments, Format, FormattableValue, Specifier};
//! use std::cmp::PartialEq;
//! use std::convert::TryInto;
//! use std::fmt;
//!
//! #[derive(Debug, PartialEq)]
//! pub enum Variant {
//!     Int(i32),
//!     Float(f64),
//! }
//! 
//! impl FormattableValue for Variant {
//!     fn supports_format(&self, spec: &Specifier) -> bool {
//!         match self {
//!             Self::Int(_) => true,
//!             Self::Float(_) => match spec.format {
//!                 Format::Display | Format::Debug | Format::LowerExp | Format::UpperExp => true,
//!                 _ => false,
//!             },
//!         }
//!     }
//! 
//!     fn fmt_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         match self {
//!             Self::Int(val) => fmt::Display::fmt(&val, f),
//!             Self::Float(val) => fmt::Display::fmt(&val, f),
//!         }
//!     }
//! 
//!     fn fmt_debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         fmt::Debug::fmt(self, f)
//!     }
//! 
//!     fn fmt_octal(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         match self {
//!             Self::Int(val) => fmt::Octal::fmt(&val, f),
//!             _ => Err(fmt::Error),
//!         }
//!     }
//! 
//!     fn fmt_lower_hex(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         match self {
//!             Self::Int(val) => fmt::LowerHex::fmt(&val, f),
//!             _ => Err(fmt::Error),
//!         }
//!     }
//! 
//!     fn fmt_upper_hex(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         match self {
//!             Self::Int(val) => fmt::UpperHex::fmt(&val, f),
//!             _ => Err(fmt::Error),
//!         }
//!     }
//! 
//!     fn fmt_binary(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         match self {
//!             Self::Int(val) => fmt::Binary::fmt(&val, f),
//!             _ => Err(fmt::Error),
//!         }
//!     }
//! 
//!     fn fmt_lower_exp(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         match self {
//!             Self::Int(val) => fmt::LowerExp::fmt(&val, f),
//!             Self::Float(val) => fmt::LowerExp::fmt(&val, f),
//!         }
//!     }
//! 
//!     fn fmt_upper_exp(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         match self {
//!             Self::Int(val) => fmt::UpperExp::fmt(&val, f),
//!             Self::Float(val) => fmt::UpperExp::fmt(&val, f),
//!         }
//!     }
//! }
//! 
//! impl TryInto<usize> for &Variant {
//!     type Error = ();
//!     fn try_into(self) -> Result<usize, Self::Error> {
//!         match self {
//!             Variant::Int(val) => (*val).try_into().map_err(|_| ()),
//!             Variant::Float(_) => Err(()),
//!         }
//!     }
//! }
//! 
//! fn main() {
//!     use std::collections::HashMap;
//! 
//!     let pos_args = [Variant::Int(42), Variant::Int(5)];
//! 
//!     let mut named_args = HashMap::new();
//!     named_args.insert("foo".to_string(), Variant::Float(42.042));
//! 
//!     let args = Arguments::parse("{:#x} [{0:<5}] {foo:.1$}", &pos_args, &named_args).unwrap();
//!     assert_eq!("0x2a [42   ] 42.04200", format!("{}", args));
//! }
//! ```

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
    /// Specifies the alignment of an argument with a specific width.
    align: Align {
        None => "",
        Left => "<",
        Center => "^",
        Right => ">",
    }

    /// Specifies whether the sign of a numeric argument should always be emitted.
    sign: Sign {
        Default => "",
        Always => "+",
    }

    /// Specifies whether to use the alternate representation for certain formats.
    repr: Repr {
        Default => "",
        Alt => "#",
    }

    /// Specifies whether a numeric argument with specific width should be padded with spaces or
    /// zeroes.
    pad: Pad {
        Space => "",
        Zero => "0",
    }

    /// Specifies whether an argument should be padded to a specific width.
    width: Width {
        Auto => "",
        AtLeast { width: usize } => "width$",
    }

    /// Specifies whether an argument should be formatted with a specific precision.
    precision: Precision {
        Auto => "",
        Exactly { precision: usize } => ".precision$",
    }

    /// Specifies how to format an argument.
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

impl Specifier {
    /// Parse a specifier from an inner specifier string.
    ///
    /// Errors if positional or named arguments are required to resolve some of its attributes
    /// like width or precision (e.g. `width$.2`).
    ///
    /// # Examples
    /// ```
    /// # use rt_format::{Align, Format, Pad, Precision, Repr, Sign, Specifier, Width};
    /// assert_eq!(
    ///     Specifier::parse("^+#8.2"),
    ///     Ok(Specifier {
    ///         align: Align::Center,
    ///         sign: Sign::Always,
    ///         repr: Repr::Alt,
    ///         pad: Pad::Space,
    ///         width: Width::AtLeast { width: 8 },
    ///         precision: Precision::Exactly { precision: 2 },
    ///         format: Format::Display,
    ///     })
    /// );
    /// assert_eq!(Specifier::parse("width$.2"), Err(()));
    /// ```
    pub fn parse(spec_str: &str) -> Result<Specifier, ()> {
        parser::parse_specifier_from_str(spec_str)
    }
}

impl Default for Specifier {
    fn default() -> Self {
        Specifier {
            align: Align::None,
            sign: Sign::Default,
            repr: Repr::Default,
            pad: Pad::Space,
            width: Width::Auto,
            precision: Precision::Auto,
            format: Format::Display,
        }
    }
}
impl fmt::Display for Specifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Specifier { align, sign, repr, pad, width, precision, format } = self;
        match align {
            Align::None => (),
            Align::Left => write!(f, "<")?,
            Align::Center => write!(f, "^")?,
            Align::Right => write!(f, ">")?,
        }
        match sign {
            Sign::Default => (),
            Sign::Always => write!(f, "+")?,
        }
        match repr {
            Repr::Default => (),
            Repr::Alt => write!(f, "#")?,
        }
        match pad {
            Pad::Space => (),
            Pad::Zero => write!(f, "0")?,
        }
        match width {
            Width::Auto => (),
            Width::AtLeast { width } => write!(f, "{}", width)?,
        }
        match precision {
            Precision::Auto => (),
            Precision::Exactly { precision } => write!(f, ".{}", precision)?,
        }
        match format {
            Format::Display => (),
            Format::Debug => write!(f, "?")?,
            Format::Octal => write!(f, "o")?,
            Format::LowerHex => write!(f, "x")?,
            Format::UpperHex => write!(f, "X")?,
            Format::Binary => write!(f, "b")?,
            Format::LowerExp => write!(f, "e")?,
            Format::UpperExp => write!(f, "E")?,
        }
        Ok(())
    }
}
