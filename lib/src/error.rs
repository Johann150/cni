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

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    ExpectedSectionEnd,
    InvalidKey,
    ExpectedKey,
    ExpectedEquals,
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
