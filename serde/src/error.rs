use std::num::{ParseFloatError, ParseIntError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub line: usize,
    pub col: usize,
    pub kind: Kind,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}:{}: {}", self.line, self.col, self.kind)
    }
}

impl From<cni_format::error::Error> for Error {
    fn from(err: cni_format::error::Error) -> Self {
        Self {
            line: err.line,
            col: err.col,
            kind: err.kind.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    // error kinds from the core library
    /// Error in the CNI syntax: expected a ']' to end the section header.
    ExpectedSectionEnd,
    /// Error in the CNI syntax: a key was formatted incorrectly
    InvalidKey,
    /// Error in the CNI syntax: a key was expected
    ExpectedKey,
    /// Error in the CNI syntax: an equals sign was expected
    ExpectedEquals,
    /// Error in the CNI syntax: unterminated raw string literal
    UnterminatedRaw,

    // later parsing errors
    /// Error in the data representation: malformed integer value
    Int(ParseIntError),
    /// Error in the data representation: malformed float value
    Float(ParseFloatError),
    /// Error in the data representation: malformed boolean value
    Bool,
    /// Error in the data representation: malformed unit value
    Unit,
    /// Error in the data representation: malformed char value
    Char,
    /// Error in the data representation: duplicate key
    DuplicateKey(String),
    /// Error in the data representation: no more value(s)
    ExpectedValues,

    /// custom error message
    Custom(String),
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedSectionEnd => write!(f, r#"expected "]""#),
            Self::InvalidKey => write!(f, "invalid key, can not start or end with a dot"),
            Self::ExpectedKey => write!(f, "expected key"),
            Self::ExpectedEquals => write!(f, r#"expected "=""#),
            Self::UnterminatedRaw => write!(f, "unterminated raw value"),

            Self::Int(err) => write!(f, "malformed integer: {}", err),
            Self::Float(err) => write!(f, "malformed float: {}", err),
            Self::Bool => write!(f, "malformed boolean"),
            Self::Unit => write!(f, "malformed unit value"),
            Self::Char => write!(f, "malformed character value"),
            Self::DuplicateKey(key) => write!(f, "key '{}' appears multiple times", key),
            Self::ExpectedValues => write!(f, "expected more values, but this is the last one"),

            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl From<cni_format::error::Kind> for Kind {
    fn from(kind: cni_format::error::Kind) -> Self {
        use cni_format::error::Kind;

        match kind {
            Kind::ExpectedSectionEnd => Self::ExpectedSectionEnd,
            Kind::InvalidKey => Self::InvalidKey,
            Kind::ExpectedKey => Self::ExpectedKey,
            Kind::ExpectedEquals => Self::ExpectedEquals,
            Kind::UnterminatedRaw => Self::UnterminatedRaw,
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T: std::string::ToString>(msg: T) -> Self {
        Error {
            line: 0,
            col: 0,
            kind: Kind::Custom(msg.to_string()),
        }
    }
}

impl std::error::Error for Error {}
