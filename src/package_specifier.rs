use std::fmt::Display;
use std::path::PathBuf;

use crate::posy::requirement::{ParseExtra, Requirement};

#[derive(Debug)]
pub enum PackageSpecifier {
    Pep508Specifier(Requirement),

    #[allow(dead_code)]
    LocalPackage(PathBuf),

    #[allow(dead_code)]
    RemotePackage(PathBuf),
}

impl PackageSpecifier {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        let maybe_requirement = Requirement::parse(value, ParseExtra::Allowed);
        if let Ok(requirement) = maybe_requirement {
            return Ok(Self::Pep508Specifier(requirement));
        }
        todo!("");
    }

    pub fn name(&self) -> String {
        match self {
            Self::Pep508Specifier(req) => req.name.normalized().to_string(),
            _ => unimplemented!(),
        }
    }
}

impl Display for PackageSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pep508Specifier(req) => req.fmt(f),
            _ => unimplemented!(),
        }
    }
}
