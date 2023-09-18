use eyre::{bail, Context, Result};
use std::fmt::Display;

use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Specifier {
    pub op: CompareOp,
    pub value: String,
}

impl Display for Specifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.op, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr, Default)]
pub struct Specifiers(pub Vec<Specifier>);

impl Display for Specifiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for spec in &self.0 {
            if !first {
                write!(f, ", ")?
            }
            first = false;
            write!(f, "{}", spec)?
        }
        Ok(())
    }
}

impl TryFrom<&str> for Specifiers {
    type Error = eyre::Report;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let specifiers_or_err = crate::posy::requirement_parser::versionspec(input);
        specifiers_or_err
            .wrap_err_with(|| format!("failed to parse versions specifiers from {:?}", input))
    }
}

impl std::convert::TryFrom<String> for Specifiers {
    type Error = eyre::Report;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        (&*s).try_into()
    }
}

impl std::str::FromStr for Specifiers {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CompareOp {
    LessThanEqual,
    StrictlyLessThan,
    NotEqual,
    Equal,
    GreaterThanEqual,
    StrictlyGreaterThan,
    Compatible,
}

impl Display for CompareOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CompareOp::*;
        write!(
            f,
            "{}",
            match self {
                LessThanEqual => "<=",
                StrictlyLessThan => "<",
                NotEqual => "!=",
                Equal => "==",
                GreaterThanEqual => ">=",
                StrictlyGreaterThan => ">",
                Compatible => "~=",
            }
        )
    }
}

impl TryFrom<&str> for CompareOp {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use CompareOp::*;
        Ok(match value {
            "==" => Equal,
            "!=" => NotEqual,
            "<=" => LessThanEqual,
            "<" => StrictlyLessThan,
            ">=" => GreaterThanEqual,
            ">" => StrictlyGreaterThan,
            "~=" => Compatible,
            "===" => bail!("'===' is not implemented"),
            _ => bail!("unrecognized operator: {:?}", value),
        })
    }
}
