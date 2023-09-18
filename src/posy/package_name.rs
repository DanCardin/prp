pub use std::collections::{HashMap, HashSet};
pub use std::fmt::Display;
pub use std::io::{Read, Seek, Write};
pub use std::rc::Rc;
pub use std::str::FromStr;

pub use derivative::Derivative;
pub use eyre::{bail, eyre, Result, WrapErr};
pub use once_cell::sync::Lazy;
pub use regex::Regex;
pub use serde::{Deserialize, Serialize};
pub use serde_with::{DeserializeFromStr, SerializeDisplay};
pub use shrinkwraprs::Shrinkwrap;
pub use tracing::{debug, info, trace, warn};
pub use url::Url;

pub trait ReadPlusSeek: Read + Seek {}
impl<T> ReadPlusSeek for T where T: Read + Seek {}

#[derive(Debug, Clone, DeserializeFromStr, Derivative)]
#[derivative(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageName {
    #[derivative(Hash = "ignore", PartialEq = "ignore", PartialOrd = "ignore")]
    as_given: String,
    normalized: String,
}

impl PackageName {
    pub fn as_given(&self) -> &str {
        &self.as_given
    }

    pub fn normalized(&self) -> &str {
        &self.normalized
    }
}

impl TryFrom<&str> for PackageName {
    type Error = eyre::Report;

    fn try_from(as_given: &str) -> Result<Self, Self::Error> {
        // https://packaging.python.org/specifications/core-metadata/#name
        static NAME_VALIDATE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i-u)^([A-Z0-9]|[A-Z0-9][A-Z0-9._-]*[A-Z0-9])$").unwrap());
        // https://www.python.org/dev/peps/pep-0503/#normalized-names
        static NAME_NORMALIZE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[-_.]").unwrap());

        if !NAME_VALIDATE.is_match(as_given) {
            return Err(eyre!("Invalid package name {:?}", as_given));
        }
        let as_given = as_given.to_owned();

        let mut normalized = NAME_NORMALIZE.replace_all(&as_given, "-").to_string();
        normalized.make_ascii_lowercase();

        Ok(PackageName {
            as_given,
            normalized,
        })
    }
}

impl std::convert::TryFrom<String> for PackageName {
    type Error = eyre::Report;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        (&*s).try_into()
    }
}

impl std::str::FromStr for PackageName {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl Serialize for PackageName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_given())
    }
}
