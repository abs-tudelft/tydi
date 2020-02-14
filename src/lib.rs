//! Tydi is an open specification for complex data structures over hardware
//! streams.
//!
//! The `tydi` crate provides an implementation of the [Tydi specification].
//!
//! # Tydi crate
//!
//! ## Modules
//! The `tydi` crate provides the following modules.
//!
//! - [`physical`]: for physical stream types.
//! - [`logical`]: for logical stream types.
//!
//! ## Features
//!
//! The `tydi` crate supports the following (non-default) features:
//!
//! - `cli`: [`tydi` command-line-interface].
//! - `generator`: [`generator`] module for generation of HDL templates.
//! - `parser`: [`parser`] module for generation of parser functions.
//!
//! # Tools
//!
//! ## `tydi` command-line-interface
//!
//! The `tydi` command-line-interface provides easy access to the available
//! tools in this crate. It can be easily installed from source using `cargo`.
//!
//! ### Install
//!
//! ```bash
//! cargo install tydi
//! ```
//!
//! ### Usage
//!
//! ```bash
//! tydi --help
//! ```
//!
//! # Examples
//!
//! ...
//!
//! # Specification
//!
//! The [Tydi specification] is available in the [Tydi book].
//!
//! [Tydi specification]: https://abs-tudelft.github.io/tydi/specification/
//! [Tydi book]: https://abs-tudelft.github.io/tydi/
//! [`physical`]: ./physical/index.html
//! [`logical`]: ./logical/index.html
//! [`generator`]: ./generator/index.html
//! [`parser`]: ./parser/index.html
//! [`tydi` command-line-interface]: #tydi-command-line-interface

// Crate utils
pub(crate) mod util;

// Core
mod error;
pub mod logical;
pub mod physical;
// pub mod streamlet;
mod traits;

// Tools
#[cfg(feature = "generator")]
pub mod generator;
// #[cfg(feature = "parser")]
// pub mod parser;

// Root re-exports
// TODO(mb): discuss
pub use error::{Error, Result};
pub use traits::{Reverse, Reversed};

// Types for positive and non-negative integers.

/// Positive integer.
pub type Positive = std::num::NonZeroU32;
/// Non-negative integer.
pub type NonNegative = u32;
/// Positive real.
pub type PositiveReal = NonZeroReal<f64>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonZeroReal<T>(T);

impl<T> NonZeroReal<T>
where
    T: Copy + Into<f64>,
{
    pub fn new(real: T) -> Result<Self> {
        if real.into() > 0. {
            Ok(NonZeroReal(real))
        } else {
            Err(Error::InvalidArgument("real must be positive".to_string()))
        }
    }
}

use std::ops::Mul;
impl<T> Mul for NonZeroReal<T>
where
    T: Copy + Mul<Output = T> + Into<f64>,
{
    type Output = NonZeroReal<T>;

    fn mul(self, other: NonZeroReal<T>) -> Self::Output {
        NonZeroReal::new(self.0 * other.0).unwrap()
    }
}

impl<T> NonZeroReal<T>
where
    T: Copy,
{
    pub fn get(&self) -> T {
        self.0
    }
}

/// Type-safe wrapper for valid names.
///
/// The following rules apply for valid names
/// - The name is non-empty
/// - The name consists of letter, number and underscores
/// - The name does not start or end with an underscore
/// - The name does not start with a digit
/// - The name does not contain double underscores
///
/// # Examples
///
/// ```rust
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    /// Constructs a new name wrapper. Returns an error when the provided name
    /// is invalid.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            Err(Error::InvalidArgument("name cannot be empty".to_string()))
        } else if name.chars().next().unwrap().is_ascii_digit() {
            Err(Error::InvalidArgument(
                "name cannot start with a digit".to_string(),
            ))
        } else if name.starts_with('_') || name.ends_with('_') {
            Err(Error::InvalidArgument(
                "name cannot start or end with an underscore".to_string(),
            ))
        } else if name.contains("__") {
            Err(Error::InvalidArgument(
                "name cannot contain two or more consecutive underscores".to_string(),
            ))
        } else if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c.eq(&'_'))
        {
            Err(Error::InvalidArgument(
                "name must consist of letters, numbers, and/or underscores".to_string(),
            ))
        } else {
            Ok(Name(name))
        }
    }
}

impl From<Name> for String {
    fn from(name: Name) -> Self {
        name.0
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

use std::ops::Deref;
impl Deref for Name {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_ref()
    }
}

use std::convert::TryFrom;
impl TryFrom<&str> for Name {
    type Error = Error;
    fn try_from(str: &str) -> Result<Self> {
        Name::new(str)
    }
}

impl TryFrom<String> for Name {
    type Error = Error;
    fn try_from(string: String) -> Result<Self> {
        Name::new(string)
    }
}

use std::str::FromStr;
impl FromStr for Name {
    type Err = Error;
    fn from_str(str: &str) -> Result<Self> {
        Name::new(str)
    }
}

impl PartialEq<String> for Name {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

use std::fmt;
impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

use std::collections::VecDeque;

/// Type-safe path for names.
///
/// Allows wrapping a set of valid names in a hierarchy.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathName(VecDeque<Name>);

use std::convert::TryInto;
impl PathName {
    pub(crate) fn new_empty() -> Self {
        PathName(VecDeque::new())
    }

    pub fn new(names: impl IntoIterator<Item = Name>) -> Self {
        PathName(names.into_iter().collect())
    }

    pub fn try_new(
        names: impl IntoIterator<Item = impl TryInto<Name, Error = Error>>,
    ) -> Result<Self> {
        Ok(PathName(
            names
                .into_iter()
                .map(|name| name.try_into())
                .collect::<Result<_>>()?,
        ))
    }
    /// Returns true if this PathName is empty (∅).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn push_back(&mut self, name: impl Into<Name>) {
        self.0.push_back(name.into());
    }
}

impl fmt::Display for PathName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = String::new();
        let mut names = self.0.iter().map(|x| x.as_ref());
        if let Some(x) = names.next() {
            result.push_str(&x);
            names.for_each(|name| {
                result.push_str("__");
                result.push_str(name);
            });
        } else {
            result.push_str("∅");
        }
        write!(f, "{}", result)
    }
}

impl<'a> IntoIterator for &'a PathName {
    type Item = &'a Name;
    type IntoIter = std::collections::vec_deque::Iter<'a, Name>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

use std::iter::FromIterator;
impl FromIterator<Name> for PathName {
    fn from_iter<I: IntoIterator<Item = Name>>(iter: I) -> Self {
        PathName(VecDeque::from_iter(iter))
    }
}

impl From<Name> for PathName {
    fn from(name: Name) -> Self {
        let mut vec = VecDeque::with_capacity(1);
        vec.push_back(name);
        PathName(vec)
    }
}

impl TryFrom<String> for PathName {
    type Error = Error;
    fn try_from(string: String) -> Result<Self> {
        let name: Name = string.try_into()?;
        Ok(PathName::from(name))
    }
}

impl TryFrom<&str> for PathName {
    type Error = Error;
    fn try_from(str: &str) -> Result<Self> {
        let name: Name = str.try_into()?;
        Ok(PathName::from(name))
    }
}
