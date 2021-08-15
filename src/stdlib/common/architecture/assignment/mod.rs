use std::convert::TryInto;
use std::fmt;

use crate::generator::common::Type;
use crate::physical::Width;
use crate::{Error, Name, Result};
use indexmap::map::IndexMap;

use self::bitvec::BitVecAssignment;

use super::declaration::ObjectDeclaration;
use super::object::ObjectType;

mod bitvec;

/// An object can be assigned a value or from another object
#[derive(Debug, Clone)]
pub enum Assignment {
    /// An object is assigned from or driven by another object
    Object(ObjectAssignment),
    /// An object is assigned a value directly
    Value(ValueAssignment),
}

/// A trait to verify whether something can be assigned
pub trait CanAssign {
    /// Verifies whether the assignment is possible
    fn can_assign(
        &self,
        assignment: &Assignment,
        to_constraint: Option<&AssignConstraint>,
    ) -> Result<()>;

    /// Verifies whether the assignment is possible when applying a constraint to the entity being assigned to
    fn can_assign_to(
        &self,
        assignment: &Assignment,
        to_constraint: &AssignConstraint,
    ) -> Result<()> {
        self.can_assign(assignment, Some(to_constraint))
    }
}

/// An object can be assigned a value or another object
#[derive(Debug, Clone)]
pub struct ObjectAssignment {
    /// The object being assigned from
    object: Box<ObjectDeclaration>,
    /// An optional constraint on the object being assigned to
    to_constraint: Option<AssignConstraint>,
    /// An optional constraint on the object being assigned from
    from_constraint: Option<AssignConstraint>,
}

impl ObjectAssignment {
    /// Returns a reference to the object being assigned from
    pub fn object(&self) -> &ObjectDeclaration {
        &self.object
    }

    pub fn assign_from(mut self, from_constraint: AssignConstraint) -> Result<Self> {
        match self.typ() {
            Type::Bit => {
                if from_range.width() == Width::Scalar {
                    self.from_range = Some(from_range);
                    Ok(self)
                } else {
                    todo!()
                }
            }
            Type::BitVec { width } => todo!(),
            Type::Record(_) => todo!(),
            Type::Union(_) => todo!(),
            Type::Array(_) => todo!(),
        }
    }

    pub fn assign_to_range(mut self, to_range: RangeConstraint) -> Result<Self> {
        match self.typ() {
            Type::Bit => {
                if to_range.width() == Width::Scalar {
                    self.to_range = Some(to_range);
                    Ok(self)
                } else {
                    Err(Error::InvalidArgument(
                        "Cannot assign a Bit to a range".to_string(),
                    ))
                }
            }
            Type::BitVec { width } => {
                if to_range.high() > *width as i32 {}
                todo!()
            }
            Type::Record(_) => todo!(),
            Type::Union(_) => todo!(),
            Type::Array(_) => todo!(),
        }
    }

    pub fn to_constraint(&self) -> &Option<AssignConstraint> {
        &self.to_constraint
    }

    pub fn from_constraint(&self) -> &Option<AssignConstraint> {
        &self.from_constraint
    }

    pub fn typ(&self) -> &ObjectType {
        self.object().typ()
    }

    pub fn finish(self) -> Result<Self> {
        /// TODO: Check whether constraints are possible
        todo!()
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

/// Assigning a value, corresponds to the Types defined in `tydi::generator::common::Type`
#[derive(Debug, Clone)]
pub enum ValueAssignment {
    /// Assigning a value to a single bit
    Bit(StdLogicValue),
    /// Assigning a value to a (part of) a bit vector
    BitVec(BitVecAssignment),
    /// Assigning one or multiple values to a Record
    Record(IndexMap<Name, ValueAssignment>),
    /// Assigning a value to a variant within a Union
    Union(Name, Box<ValueAssignment>),
    // TODO: Array
}

/// A VHDL assignment constraint
#[derive(Debug, Clone)]
pub enum AssignConstraint {
    /// The most common kind of constraint, a specific range or index
    Range(RangeConstraint),
    /// The field of a record
    Name(Name),
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

impl fmt::Display for RangeConstraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RangeConstraint::To { start, end } => write!(f, "({} to {})", start, end),
            RangeConstraint::Downto { start, end } => write!(f, "({} downto {})", start, end),
            RangeConstraint::Index(index) => write!(f, "({})", index),
        }
    }
}

impl RangeConstraint {
    /// Create a `RangeConstraint::To` and ensure correctness (end > start)
    pub fn to(start: i32, end: i32) -> crate::Result<RangeConstraint> {
        if start > end {
            Err(Error::InvalidArgument(format!(
                "{} > {}!\nStart cannot be greater than end when constraining a range [start] to [end]",
                start, end
            )))
        } else {
            Ok(RangeConstraint::To { start, end })
        }
    }

    /// Create a `RangeConstraint::DownTo` and ensure correctness (start > end)
    pub fn downto(start: i32, end: i32) -> crate::Result<RangeConstraint> {
        if end > start {
            Err(Error::InvalidArgument(format!(
                "{} > {}!\nEnd cannot be greater than start when constraining a range [start] downto [end]",
                end, start
            )))
        } else {
            Ok(RangeConstraint::Downto { start, end })
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

    /// Returns the width of the range
    pub fn width_u32(&self) -> u32 {
        match self.width() {
            Width::Scalar => 1,
            Width::Vector(width) => width,
        }
    }

    /// Returns the greatest index within the range constraint
    pub fn high(&self) -> i32 {
        match self {
            RangeConstraint::To { start: _, end } => *end,
            RangeConstraint::Downto { start, end: _ } => *start,
            RangeConstraint::Index(index) => *index,
        }
    }

    /// Returns the smallest index within the range constraint
    pub fn low(&self) -> i32 {
        match self {
            RangeConstraint::To { start, end: _ } => *start,
            RangeConstraint::Downto { start: _, end } => *end,
            RangeConstraint::Index(index) => *index,
        }
    }
}
