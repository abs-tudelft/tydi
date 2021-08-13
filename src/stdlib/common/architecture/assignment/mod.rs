use std::convert::TryInto;

use crate::generator::common::Type;
use crate::physical::Width;
use crate::{Error, Name};
use indexmap::map::IndexMap;

use self::bitvec::BitVecAssignment;

mod bitvec;

/// An object can be assigned a value or from another object
#[derive(Debug, Clone)]
pub enum Assignment {
    /// An object is assigned from or driven by another object
    Object(ObjectAssignment),
    /// An object is assigned a value directly
    Value(ValueAssignment),
}

/// A trait to verify whether something can be assigned from another Type
pub trait CanAssignFrom {
    fn can_assign_from(typ: Type) -> bool;
}

/// An object can be assigned a value or another object
#[derive(Debug, Clone)]
pub struct ObjectAssignment {
    /// The object's identifier
    identifier: Name,
    /// The object's type
    typ: Type,
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
