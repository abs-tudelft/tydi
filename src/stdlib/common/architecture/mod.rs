use std::convert::TryFrom;
use std::convert::TryInto;

use indexmap::IndexMap;

use crate::{
    generator::{
        common::{Component, Package, Type},
        vhdl::{ListUsings, Usings},
    },
    physical::Width,
    Identify, Name,
};
use crate::{Error, Result};

use super::entity::Entity;

mod declaration;
mod impls;

// TODO: Figure this out, either make it a struct with specific contents, or a trait for something else to implement?
/// Architecture statement.
#[derive(Debug, Clone)]
pub struct ArchitectureStatement {}

// NOTE: One of the main things to consider is probably how to handle multiple element lanes. Probably as a check on the number of lanes,
// then wrapping in a generate statement. Need to consider indexes at that point.
// This'd be easier if I simply always made it an array, even when the number of lanes is 1, but that gets real ugly, real fast.

/// Architecture declarations.
#[derive(Debug, Clone)]
pub struct ArchitectureDeclarations {}

/// An architecture
#[derive(Debug)]
pub struct Architecture {
    /// Name of the architecture
    identifier: Name,
    /// Entity which this architecture is for
    entity: Entity,
    /// Additional usings beyond the Package and those within it
    usings: Usings,
    /// Documentation.
    doc: Option<String>,
}

impl Architecture {
    /// Create the architecture based on a component contained within a package, assuming the library (project) is "work" and the architecture's identifier is "Behavioral"
    pub fn new_default(package: Package, component_id: Name) -> Result<Architecture> {
        Architecture::new(
            Name::try_new("work")?,
            Name::try_new("Behavioral")?,
            package,
            component_id,
        )
    }

    /// Create the architecture based on a component contained within a package, specify the library (project) in which the package is contained
    pub fn new(
        library_id: Name,
        identifier: Name,
        package: Package,
        component_id: Name,
    ) -> Result<Architecture> {
        if let Some(component) = package
            .components
            .iter()
            .find(|x| component_id == *x.identifier())
        {
            let mut usings = package.list_usings()?;
            usings.add_using(library_id, format!("{}.all", package.identifier));
            Ok(Architecture {
                identifier,
                entity: Entity::from(component.clone()),
                usings: usings,
                doc: None,
            })
        } else {
            Err(Error::InvalidArgument(format!(
                "Identifier \"{}\" does not exist in this package",
                component_id
            )))
        }
    }

    /// Add additional usings which weren't already part of the package
    pub fn add_using(&mut self, library: Name, using: String) -> bool {
        self.usings.add_using(library, using)
    }

    /// Return this architecture with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set the documentation of this architecture.
    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }
}

/// Possible values which can be assigned to std_logic
#[derive(Debug, Clone)]
pub enum StdLogicValue {
    /// Uninitialized, 'U'
    U,
    /// Unknown, 'X',
    X,
    /// Logic, '0' or '1'
    Logic(bool),
    /// High Impedance, 'Z'
    Z,
    /// Weak signal (either '0' or '1'), 'W'
    W,
    /// Weak signal (likely '0'), 'L'
    L,
    /// Weak signal (likely '1'), 'H'
    H,
    /// Don't care, '-'
    DontCare,
}

/// Corresponds to the Types defined in `tydi::generator::common::Type`
#[derive(Debug, Clone)]
pub enum ValueAssignment {
    Bit(StdLogicValue),
    BitVec(Vec<char>),
    Record(IndexMap<Name, ValueAssignment>),
    Union(Name, Box<ValueAssignment>),
}

/// A VHDL range constraint
#[derive(Debug, Clone)]
pub enum RangeConstraint {
    /// A range [start] to [end]
    To { start: i32, end: i32 },
    /// A range [start] downto [end]
    Downto { start: i32, end: i32 },
    /// An index within a range
    Index(i32),
}

impl RangeConstraint {
    /// Create a `RangeConstraint::To` and ensure correctness (end > start)
    pub fn to(start: i32, end: i32) -> Result<RangeConstraint> {
        if end >= start {
            Ok(RangeConstraint::To { start, end })
        } else {
            Err(Error::InvalidArgument(format!(
                "{} > {}!\nStart cannot be greater than end when constraining a range [start] to [end]",
                start, end
            )))
        }
    }

    /// Create a `RangeConstraint::DownTo` and ensure correctness (start > end)
    pub fn downto(start: i32, end: i32) -> Result<RangeConstraint> {
        if start >= end {
            Ok(RangeConstraint::Downto { start, end })
        } else {
            Err(Error::InvalidArgument(format!(
                "{} > {}!\nEnd cannot be greater than start when constraining a range [start] downto [end]",
                start, end
            )))
        }
    }

    /// Returns the width of the range
    pub fn width(&self) -> Width {
        match self {
            RangeConstraint::To { start, end } => Width::Vector((end - start).try_into().unwrap()),
            RangeConstraint::Downto { start, end } => {
                Width::Vector((start - end).try_into().unwrap())
            }
            RangeConstraint::Index(_) => Width::Scalar,
        }
    }

    /// Returns the greatest index within the range constraint
    pub fn max_index(&self) -> i32 {
        match self {
            RangeConstraint::To { start: _, end } => *end,
            RangeConstraint::Downto { start, end: _ } => *start,
            RangeConstraint::Index(index) => *index,
        }
    }

    /// Returns the smallest index within the range constraint
    pub fn min_index(&self) -> i32 {
        match self {
            RangeConstraint::To { start, end: _ } => *start,
            RangeConstraint::Downto { start: _, end } => *end,
            RangeConstraint::Index(index) => *index,
        }
    }
}

/// A struct for describing an assignment to a bit vector
#[derive(Debug, Clone)]
pub struct BitVecAssignment {
    /// When range_constraint is None, the entire range is assigned
    range_constraint: Option<RangeConstraint>,
    /// The values assigned to the range
    value: Vec<StdLogicValue>,
}

impl BitVecAssignment {
    /// Create a new index-based assignment of a bit vector
    pub fn index(index: i32, value: StdLogicValue) -> BitVecAssignment {
        BitVecAssignment {
            range_constraint: Some(RangeConstraint::Index(index)),
            value: vec![value],
        }
    }

    /// Create a new downto-range assignment of a bit vector
    pub fn downto(start: i32, end: i32, value: Vec<StdLogicValue>) -> Result<BitVecAssignment> {
        if usize::try_from(start - end)
            .map(|w| w == value.len())
            .unwrap_or(false)
        {
            Ok(BitVecAssignment {
                range_constraint: Some(RangeConstraint::downto(start, end)?),
                value,
            })
        } else {
            Err(Error::InvalidArgument(format!("Values do not match")))
        }
    }

    /// Create a new downto-range assignment of a bit vector
    pub fn to(start: i32, end: i32, value: Vec<StdLogicValue>) -> Result<BitVecAssignment> {
        if usize::try_from(end - start)
            .map(|w| w == value.len())
            .unwrap_or(false)
        {
            Ok(BitVecAssignment {
                range_constraint: Some(RangeConstraint::to(start, end)?),
                value,
            })
        } else {
            Err(Error::InvalidArgument(format!("Values do not match")))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::common::convert::Packify;

    use super::*;

    pub fn test_package() -> Package {
        let (_, streamlet) = crate::parser::nom::streamlet(
            "Streamlet test (a : in Stream<Bits<1>>, b : out Stream<Bits<2>, d=2>)",
        )
        .unwrap();
        let lib = crate::design::library::Library::try_new(
            Name::try_new("test").unwrap(),
            vec![],
            vec![streamlet],
        );
        let lib: crate::generator::common::Package = lib.unwrap().fancy();
        lib
    }

    #[test]
    fn new_architecture() {
        let package = test_package();
        let architecture = Architecture::new_default(package, Name::try_new("test").unwrap());

        print!("{:?}", architecture);
    }
}
