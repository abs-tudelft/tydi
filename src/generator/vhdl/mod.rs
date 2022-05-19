//! VHDL back-end.
//!
//! This module contains functionality to convert hardware defined in the common hardware
//! representation to VHDL source files.

use crate::design::Project;
use crate::generator::common::*;
use crate::generator::GenerateProject;
use crate::{Error, Result, Reversed};
use log::debug;
use std::path::Path;

use crate::cat;
use crate::generator::common::convert::Packify;
use crate::traits::Identify;
use std::str::FromStr;
#[cfg(feature = "cli")]
use structopt::StructOpt;

mod impls;

/// Generate trait for generic VHDL declarations.
pub trait Declare {
    /// Generate a VHDL declaration from self.
    fn declare(&self) -> Result<String>;
}

/// Generate trait for VHDL type declarations.
pub trait DeclareType {
    /// Generate a VHDL declaration from self.
    fn declare(&self, is_root_type: bool) -> Result<String>;
}

/// Generate trait for VHDL package declarations.
pub trait DeclareLibrary {
    /// Generate a VHDL declaration from self.
    fn declare(&self, abstraction: AbstractionLevel) -> Result<String>;
}

/// Generate trait for VHDL identifiers.
pub trait VHDLIdentifier {
    /// Generate a VHDL identifier from self.
    fn vhdl_identifier(&self) -> Result<String>;
}

/// Analyze trait for VHDL objects.
pub trait Analyze {
    /// List all record types used.
    fn list_record_types(&self) -> Vec<Type>;
}

/// Abstraction levels
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "cli", derive(StructOpt))]
pub enum AbstractionLevel {
    Canonical,
    Fancy,
}

impl Default for AbstractionLevel {
    fn default() -> Self {
        AbstractionLevel::Fancy
    }
}

impl FromStr for AbstractionLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "canon" => Ok(AbstractionLevel::Canonical),
            "fancy" => Ok(AbstractionLevel::Fancy),
            _ => Err(Error::InvalidArgument(s.to_string())),
        }
    }
}

/// VHDL back-end configuration parameters.
#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(StructOpt))]
pub struct VHDLConfig {
    /// Abstraction level of generated files.
    /// Possible options: canonical, fancy.
    ///   canonical: generates the canonical Tydi representation of streamlets as components in a
    ///              package.
    ///   fancy: generates the canonical components that wrap a more user-friendly version for the
    ///          user to implement.
    #[cfg_attr(feature = "cli", structopt(short, long))]
    abstraction: Option<AbstractionLevel>,

    /// Suffix of generated files. Default = "gen", such that
    /// generated files are named <name>.gen.vhd.
    #[cfg_attr(feature = "cli", structopt(short, long))]
    suffix: Option<String>,
}

impl VHDLConfig {
    pub fn abstraction(&self) -> AbstractionLevel {
        self.abstraction.unwrap_or_default()
    }
}

impl Default for VHDLConfig {
    fn default() -> Self {
        VHDLConfig {
            suffix: Some("gen".to_string()),
            abstraction: Some(AbstractionLevel::Canonical),
        }
    }
}

/// A configurable VHDL back-end entry point.
#[derive(Default)]
pub struct VHDLBackEnd {
    /// Configuration for the VHDL back-end.
    config: VHDLConfig,
}

impl VHDLBackEnd {
    pub fn config(&self) -> &VHDLConfig {
        &self.config
    }
}

impl From<VHDLConfig> for VHDLBackEnd {
    fn from(config: VHDLConfig) -> Self {
        VHDLBackEnd { config }
    }
}

impl GenerateProject for VHDLBackEnd {
    fn generate(&self, project: &Project, path: impl AsRef<Path>) -> Result<()> {
        // Create the project directory.
        let mut dir = path.as_ref().to_path_buf();
        dir.push(project.identifier());
        std::fs::create_dir_all(dir.as_path())?;

        for lib in project.libraries() {
            let mut pkg = dir.clone();
            pkg.push(format!("{}_pkg", lib.identifier()));
            pkg.set_extension(match self.config.suffix.clone() {
                None => "vhd".to_string(),
                Some(s) => format!("{}.vhd", s),
            });
            std::fs::write(
                pkg.as_path(),
                match self.config().abstraction() {
                    AbstractionLevel::Canonical => lib.canonical(),
                    AbstractionLevel::Fancy => lib.fancy(),
                }
                .declare()?,
            )?;
            debug!("Wrote {}.", pkg.as_path().to_str().unwrap_or(""));
        }
        Ok(())
    }
}

/// Trait used to split types, ports, and record fields into a VHDL-friendly versions, since VHDL
/// does not support bundles of wires with opposite directions.
trait Split {
    /// Split up self into a (downstream/forward, upstream/reverse) version, if applicable.
    fn split(&self) -> (Option<Self>, Option<Self>)
    where
        Self: Sized;
}

impl Split for Type {
    fn split(&self) -> (Option<Self>, Option<Self>) {
        match self {
            Type::Record(rec) => {
                let (down_rec, up_rec) = rec.split();
                (down_rec.map(Type::Record), up_rec.map(Type::Record))
            }
            _ => (Some(self.clone()), None),
        }
    }
}

impl Split for Field {
    fn split(&self) -> (Option<Self>, Option<Self>) {
        // Split the inner type.
        let (down_type, up_type) = self.typ().split();

        let result = (
            down_type.map(|t| Field::new(self.identifier(), t, false)),
            up_type.map(|t| Field::new(self.identifier(), t, false)),
        );

        if self.is_reversed() {
            // If this field itself is reversed, swap the result of splitting the field type.
            (result.1, result.0)
        } else {
            result
        }
    }
}

impl Split for Record {
    fn split(&self) -> (Option<Self>, Option<Self>) {
        let mut down_rec = Record::new_empty(self.identifier());
        let mut up_rec = Record::new_empty(self.identifier());

        for f in self.fields() {
            let (down_field, up_field) = f.split();
            if let Some(df) = down_field {
                down_rec.insert(df)
            };
            if let Some(uf) = up_field {
                up_rec.insert(uf)
            };
        }

        let f = |r: Record| if r.is_empty() { None } else { Some(r) };

        (f(down_rec), f(up_rec))
    }
}

impl Split for Port {
    fn split(&self) -> (Option<Self>, Option<Self>) {
        let (type_down, type_up) = self.typ().split();
        (
            type_down.map(|t| {
                Port::new(
                    cat!(self.identifier(), "dn"),
                    self.mode(),
                    match t {
                        Type::Record(r) => Type::Record(r.append_name_nested("dn")),
                        _ => t,
                    },
                )
            }),
            type_up.map(|t| {
                Port::new(
                    cat!(self.identifier(), "up"),
                    self.mode().reversed(),
                    match t {
                        Type::Record(r) => Type::Record(r.append_name_nested("up")),
                        _ => t,
                    },
                )
            }),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Reversed;
    use std::fs;

    #[test]
    fn split_primitive() {
        assert_eq!(Type::bitvec(3).split(), (Some(Type::bitvec(3)), None));
    }

    #[test]
    fn split_field() {
        let f0 = Field::new("test", Type::bitvec(3), false);
        assert_eq!(f0.split(), (Some(f0), None));

        let f1 = Field::new("test", Type::bitvec(3), true);
        assert_eq!(f1.split(), (None, Some(f1.reversed())));
    }

    #[test]
    fn split_simple_rec() {
        let rec = Type::record(
            "ra",
            vec![
                Field::new("fc", Type::Bit, false),
                Field::new("fd", Type::Bit, true),
            ],
        );

        assert_eq!(
            rec.split().0.unwrap(),
            Type::record("ra", vec![Field::new("fc", Type::Bit, false)])
        );

        assert_eq!(
            rec.split().1.unwrap(),
            Type::record("ra", vec![Field::new("fd", Type::Bit, false)])
        );
    }

    #[test]
    fn split_nested_rec() {
        let rec = Type::record(
            "test",
            vec![
                Field::new(
                    "fa",
                    Type::record(
                        "ra",
                        vec![
                            Field::new("fc", Type::Bit, false),
                            Field::new("fd", Type::Bit, true),
                        ],
                    ),
                    false,
                ),
                Field::new(
                    "fb",
                    Type::record(
                        "rb",
                        vec![
                            Field::new("fe", Type::Bit, false),
                            Field::new("ff", Type::Bit, true),
                        ],
                    ),
                    true,
                ),
            ],
        );

        assert_eq!(
            rec.split().0.unwrap(),
            Type::record(
                "test",
                vec![
                    Field::new(
                        "fa",
                        Type::record("ra", vec![Field::new("fc", Type::Bit, false)]),
                        false
                    ),
                    Field::new(
                        "fb",
                        Type::record("rb", vec![Field::new("ff", Type::Bit, false)]),
                        false
                    )
                ]
            )
        );

        assert_eq!(
            rec.split().1.unwrap(),
            Type::record(
                "test",
                vec![
                    Field::new(
                        "fa",
                        Type::record("ra", vec![Field::new("fd", Type::Bit, false)]),
                        false
                    ),
                    Field::new(
                        "fb",
                        Type::record("rb", vec![Field::new("fe", Type::Bit, false)]),
                        false
                    )
                ]
            )
        );
    }

    #[test]
    fn split_port() {
        let (dn, up) = Port::new_documented(
            "test",
            Mode::Out,
            Type::record(
                "test",
                vec![
                    Field::new("a", Type::Bit, false),
                    Field::new("b", Type::Bit, true),
                ],
            ),
            None,
        )
        .split();

        assert_eq!(
            dn,
            Some(Port::new_documented(
                "test_dn",
                Mode::Out,
                Type::record("test_dn", vec![Field::new("a", Type::Bit, false)]),
                None
            ))
        );

        assert_eq!(
            up,
            Some(Port::new_documented(
                "test_up",
                Mode::In,
                Type::record("test_up", vec![Field::new("b", Type::Bit, false)]),
                None
            ))
        );
    }

    #[test]
    fn backend() -> Result<()> {
        let v = VHDLBackEnd::default();

        let tmpdir = tempfile::tempdir()?;
        let path = tmpdir.path().join("__test");

        assert!(v
            .generate(&crate::design::project::tests::proj::empty_proj(), &path)
            .is_ok());

        // Check if files were correctly generated.
        assert!(fs::metadata(&path).is_ok());
        assert!(fs::metadata(&path.join("proj")).is_ok());
        assert!(fs::metadata(&path.join("proj/lib_pkg.gen.vhd")).is_ok());

        Ok(())
    }
}
