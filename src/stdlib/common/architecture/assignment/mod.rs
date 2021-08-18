use std::convert::{TryFrom, TryInto};
use std::fmt;

use indexmap::map::IndexMap;

use array_assignment::ArrayAssignment;

use crate::generator::common::Type;
use crate::physical::Width;
use crate::{Error, Name, Result};

use super::declaration::ObjectDeclaration;
use super::object::ObjectType;

use self::bitvec::BitVecAssignment;

pub mod array_assignment;
pub mod assign;
pub mod assignment_from;
pub mod bitvec;
pub mod declare;

pub trait Assign {
    fn assign(&self, assignment: Assignment) -> Result<AssignedObject>;
}

/// Describing a specific object being assigned with something
#[derive(Debug, Clone)]
pub struct AssignedObject {
    object: ObjectDeclaration,
    assignment: Assignment,
}

impl AssignedObject {
    pub fn new(object: ObjectDeclaration, assignment: Assignment) -> AssignedObject {
        AssignedObject { object, assignment }
    }

    pub fn object(&self) -> &ObjectDeclaration {
        &self.object
    }

    pub fn assignment(&self) -> &Assignment {
        &self.assignment
    }
}

/// An object can be assigned a value or from another object
#[derive(Debug, Clone)]
pub enum Assignment {
    /// An object is assigned from or driven by another object
    Object(ObjectAssignment),
    /// An object is assigned a value directly, or completely filled
    Direct(DirectAssignment),
}

impl Assignment {
    pub fn direct_record(record: IndexMap<String, Assignment>) -> Assignment {
        Assignment::Direct(DirectAssignment::Record(record))
    }

    pub fn direct_union(name: String, assignment: Assignment) -> Assignment {
        Assignment::Direct(DirectAssignment::Union(name, Box::new(assignment)))
    }
}

/// An object can be assigned a value or another object
#[derive(Debug, Clone)]
pub struct ObjectAssignment {
    /// The object being assigned from
    object: Box<ObjectDeclaration>,
    /// Optional selections on the object being assigned to, representing nested selections
    to_field: Vec<FieldSelection>,
    /// Optional selections on the object being assigned from, representing nested selections
    from_field: Vec<FieldSelection>,
}

impl ObjectAssignment {
    /// Returns a reference to the object being assigned from
    pub fn object(&self) -> &ObjectDeclaration {
        &self.object
    }

    /// Select fields from the object being assigned
    pub fn assign_from(mut self, from_field: FieldSelection) -> Result<Self> {
        match self.typ() {
            ObjectType::Bit => todo!(),
            ObjectType::Array(_) => todo!(),
            ObjectType::Record(_) => todo!(),
        }
    }

    pub fn assign_to(mut self, to_field: FieldSelection) -> Self {
        self.to_field.push(to_field);
        self
    }

    pub fn to_field(&self) -> &Vec<FieldSelection> {
        &self.to_field
    }

    pub fn from_field(&self) -> &Vec<FieldSelection> {
        &self.from_field
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

impl StdLogicValue {
    pub fn from_char(val: char) -> Result<StdLogicValue> {
        match val {
            'U' => Ok(StdLogicValue::U),
            'X' => Ok(StdLogicValue::X),
            '1' => Ok(StdLogicValue::Logic(true)),
            '0' => Ok(StdLogicValue::Logic(false)),
            'Z' => Ok(StdLogicValue::Z),
            'W' => Ok(StdLogicValue::W),
            'L' => Ok(StdLogicValue::L),
            'H' => Ok(StdLogicValue::H),
            '-' => Ok(StdLogicValue::DontCare),
            _ => Err(Error::InvalidArgument(format!(
                "Unsupported std_logic value {}",
                val
            ))),
        }
    }
}

impl fmt::Display for StdLogicValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            StdLogicValue::U => "U",
            StdLogicValue::X => "X",
            StdLogicValue::Logic(value) => {
                if *value {
                    "1"
                } else {
                    "0"
                }
            }
            StdLogicValue::Z => "Z",
            StdLogicValue::W => "W",
            StdLogicValue::L => "L",
            StdLogicValue::H => "H",
            StdLogicValue::DontCare => "-",
        };
        write!(f, "{}", symbol)
    }
}

/// Directly assigning a value or an entire Record, corresponds to the Types defined in `tydi::generator::common::Type`
#[derive(Debug, Clone)]
pub enum DirectAssignment {
    /// Assigning a value to a single bit
    Bit(StdLogicValue),
    /// Assigning a value to a (part of) a bit vector
    BitVec(BitVecAssignment),
    /// Assigning one or multiple values to a Record
    Record(IndexMap<String, Assignment>),
    /// Assigning a value to a variant within a Union
    Union(String, Box<Assignment>),
    /// Assigning one or more values or objects directly to an array (may overlap with ObjectAssignment)
    Array(ArrayAssignment),
}

/// A VHDL assignment constraint
#[derive(Debug, Clone)]
pub enum FieldSelection {
    /// The most common kind of constraint, a specific range or index
    Range(RangeConstraint),
    /// The field of a record
    Name(String),
}

impl fmt::Display for FieldSelection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FieldSelection::Range(range) => range.fmt(f),
            FieldSelection::Name(name) => write!(f, ".{}", name),
        }
    }
}

impl FieldSelection {
    pub fn to(start: i32, end: i32) -> Result<FieldSelection> {
        Ok(FieldSelection::Range(RangeConstraint::to(start, end)?))
    }

    pub fn downto(start: i32, end: i32) -> Result<FieldSelection> {
        Ok(FieldSelection::Range(RangeConstraint::downto(start, end)?))
    }

    pub fn index(index: i32) -> FieldSelection {
        FieldSelection::Range(RangeConstraint::Index(index))
    }

    pub fn name(name: &str) -> Result<FieldSelection> {
        Ok(FieldSelection::Name(name.to_string()))
    }
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
            RangeConstraint::To { start, end } => {
                Width::Vector((1 + end - start).try_into().unwrap())
            }
            RangeConstraint::Downto { start, end } => {
                Width::Vector((1 + start - end).try_into().unwrap())
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
