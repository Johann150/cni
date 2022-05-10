/// Convenience type for returning a Result that uses the Error struct
/// from this module.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that occurred while parsing the CNI syntax.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
	/// Line number on which the error occured, counting from 1.
    pub line: usize,
    /// Column on whicht the error "started", counting from 1.
    pub col: usize,
	/// The type of error that occured.
    pub kind: Kind,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}:{}: {}", self.line, self.col, self.kind)
    }
}

/// A type of error that may occur.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
	/// Syntax error: The end of a section header was expected (a closing bracket)
    ExpectedSectionEnd,
    /// Syntax error: A key (may be a section heading) was malformed.
    InvalidKey,
    /// Syntax error: A key was expected but missing.
    ExpectedKey,
    /// Syntax error: An equal sign was expected but missing.
    ExpectedEquals,
    /// Syntax error: A raw string is not terminated properly.
    UnterminatedRaw,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ExpectedSectionEnd => r#"expected "]""#,
                Self::InvalidKey => "invalid key, can not start or end with a dot",
                Self::ExpectedKey => "expected key",
                Self::ExpectedEquals => r#"expected "=""#,
                Self::UnterminatedRaw => "unterminated raw value",
            }
        )
    }
}
