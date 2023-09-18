use std::fmt::Display;

use crate::posy::specifier::CompareOp;
use eyre::{bail, Context, Result};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use shrinkwraprs::Shrinkwrap;

use super::package_name::PackageName;
use super::specifier::Specifiers;

// There are two kinds of special exact version constraints that aren't often
// used, and whose semantics are a bit unclear:
//
//  === "some string"
//  @ some_url
//
// Not sure if we should bother supporting them. For === they're easy to parse
// and represent (same as all the other binary comparisons), but I don't know
// what the semantics is, b/c we fully parse all versions. PEP 440 says "The
// primary use case ... is to allow for specifying a version which cannot
// otherwise by represented by this PEP". Maybe if we find ourselves supporting
// LegacyVersion-type versions, we should add this then? Though even then, I'm not sure
// we can convince pubgrub to handle it.
//
// If we do want to parse @ syntax, the problem is more: how do we represent
// them? Because it *replaces* version constraints, so I guess inside the
// Requirement object we'd need something like:
//
//   enum Specifiers {
//      Direct(Url),
//      Index(Vec<Specifier>),
//   }
//
// ? But then that complexity propagates through to everything that uses
// Requirements.
//
// Also, I don't think @ is allowed in public indexes like PyPI?
//
// NB: if we do decide to handle '@', then PEP 508 includes an entire copy of
// (some version of) the standard URL syntax. We don't want to do that, both
// because it's wildly more complicated than required, and because there are
// >3 different standards purpoting to define URL syntax and we don't want to
// take sides. But! The 'packaging' module just does
//
//    URI = Regex(r"[^ ]+")("url")
//
// ...so we can just steal some version of that.
//
// For resolving, we can treat it as a magic package that provides/depends on the
// version it declares, so it can satisfy other dependencies that use the name or
// versions.

pub mod marker {
    use std::collections::HashMap;
    use std::{borrow::Borrow, hash::Hash};

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Value {
        Variable(String),
        Literal(String),
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Op {
        Compare(CompareOp),
        In,
        NotIn,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum EnvMarkerExpr {
        And(Box<EnvMarkerExpr>, Box<EnvMarkerExpr>),
        Or(Box<EnvMarkerExpr>, Box<EnvMarkerExpr>),
        Operator { op: Op, lhs: Value, rhs: Value },
    }

    pub trait Env {
        fn get_marker_var(&self, var: &str) -> Option<&str>;
    }

    impl<T: Borrow<str> + Eq + Hash> Env for HashMap<T, T> {
        fn get_marker_var(&self, var: &str) -> Option<&str> {
            self.get(var).map(|s| s.borrow())
        }
    }

    impl Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Value::Variable(var) => write!(f, "{}", var),
                Value::Literal(literal) => {
                    if literal.contains('"') {
                        write!(f, "'{}'", literal)
                    } else {
                        write!(f, "\"{}\"", literal)
                    }
                }
            }
        }
    }

    impl Display for EnvMarkerExpr {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                EnvMarkerExpr::And(lhs, rhs) => write!(f, "({} and {})", lhs, rhs)?,
                EnvMarkerExpr::Or(lhs, rhs) => write!(f, "({} or {})", lhs, rhs)?,
                EnvMarkerExpr::Operator { op, lhs, rhs } => write!(
                    f,
                    "{} {} {}",
                    lhs,
                    match op {
                        Op::Compare(compare_op) => compare_op.to_string(),
                        Op::In => "in".to_string(),
                        Op::NotIn => "not in".to_string(),
                    },
                    rhs,
                )?,
            }
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, SerializeDisplay, DeserializeFromStr)]
pub struct StandaloneMarkerExpr(pub marker::EnvMarkerExpr);

impl Display for StandaloneMarkerExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for StandaloneMarkerExpr {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let expr = crate::posy::requirement_parser::marker(value, ParseExtra::NotAllowed)
            .wrap_err_with(|| format!("Failed parsing env marker expression {:?}", value))?;
        Ok(StandaloneMarkerExpr(expr))
    }
}

impl std::convert::TryFrom<String> for StandaloneMarkerExpr {
    type Error = eyre::Report;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        (&*s).try_into()
    }
}

impl std::str::FromStr for StandaloneMarkerExpr {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseExtra {
    Allowed,
    NotAllowed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    pub name: PackageName,
    pub extras: Vec<PackageName>,
    pub specifiers: Specifiers,
    pub env_marker_expr: Option<marker::EnvMarkerExpr>,
}

impl Requirement {
    pub fn parse(input: &str, parse_extra: ParseExtra) -> Result<Requirement> {
        let req = crate::posy::requirement_parser::requirement(input, parse_extra)
            .wrap_err_with(|| format!("Failed parsing requirement string {:?})", input))?;
        Ok(req)
    }
}

impl Display for Requirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.as_given())?;
        if !self.extras.is_empty() {
            write!(f, "[")?;
            let mut first = true;
            for extra in &self.extras {
                if !first {
                    write!(f, ",")?;
                }
                first = false;
                write!(f, "{}", extra.as_given())?;
            }
            write!(f, "]")?;
        }
        if !self.specifiers.0.is_empty() {
            write!(f, " {}", self.specifiers)?;
        }
        if let Some(env_marker) = &self.env_marker_expr {
            write!(f, "; {}", env_marker)?;
        }
        Ok(())
    }
}

#[derive(Shrinkwrap, Debug, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay)]
pub struct PackageRequirement(Requirement);

impl Display for PackageRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<&str> for PackageRequirement {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(PackageRequirement(Requirement::parse(
            value,
            ParseExtra::Allowed,
        )?))
    }
}

impl std::convert::TryFrom<String> for PackageRequirement {
    type Error = eyre::Report;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        (&*s).try_into()
    }
}

impl std::str::FromStr for PackageRequirement {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

#[derive(Shrinkwrap, Debug, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay)]
pub struct UserRequirement(Requirement);

impl Display for UserRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<&str> for UserRequirement {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(UserRequirement(Requirement::parse(
            value,
            ParseExtra::NotAllowed,
        )?))
    }
}

impl std::convert::TryFrom<String> for UserRequirement {
    type Error = eyre::Report;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        (&*s).try_into()
    }
}

impl std::str::FromStr for UserRequirement {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

#[derive(Shrinkwrap, Debug, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay)]
pub struct PythonRequirement(Requirement);

impl Display for PythonRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<Requirement> for PythonRequirement {
    type Error = eyre::Report;

    fn try_from(r: Requirement) -> Result<Self, Self::Error> {
        if !r.extras.is_empty() {
            bail!("can't have extras on python requirement {}", r);
        }
        if r.env_marker_expr.is_some() {
            bail!(
                "can't have env marker restrictions on python requirement {}",
                r
            );
        }
        Ok(PythonRequirement(r))
    }
}

impl TryFrom<&str> for PythonRequirement {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let r = Requirement::parse(value, ParseExtra::NotAllowed)?;
        r.try_into()
    }
}

impl std::str::FromStr for PythonRequirement {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}
