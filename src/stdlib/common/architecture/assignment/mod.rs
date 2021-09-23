use std::convert::TryInto;
use std::fmt;

use indexmap::map::IndexMap;

use array_assignment::ArrayAssignment;

use crate::physical::Width;
use crate::{Document, Error, Result};

use super::declaration::{ObjectDeclaration};
use super::object::ObjectType;

use self::bitvec::BitVecValue;

pub mod array_assignment;
pub mod assign;
pub mod assignment_from;
pub mod bitvec;
pub mod declare;
pub mod flatten;
pub mod impls;

pub trait Assign {
    fn assign(&self, assignment: &(impl Into<Assignment> + Clone)) -> Result<AssignDeclaration>;
}

/// Describing the declaration of an assignment
#[derive(Debug, Clone)]
pub struct AssignDeclaration {
    /// The declared object being assigned
    object: ObjectDeclaration,
    /// The assignment to the declared object
    assignment: Assignment,
    doc: Option<String>,
}

impl AssignDeclaration {
    pub fn new(object: ObjectDeclaration, assignment: Assignment) -> AssignDeclaration {
        AssignDeclaration {
            object,
            assignment,
            doc: None,
        }
    }

    pub fn object(&self) -> &ObjectDeclaration {
        &self.object
    }

    pub fn assignment(&self) -> &Assignment {
        &self.assignment
    }

    /// The object declaration with any field selections on it
    pub fn object_string(&self) -> String {
        let mut result = self.object().identifier().to_string();
        for field in self.assignment().to_field() {
            result.push_str(&field.to_string());
        }
        result
    }

    /// Return this assignment declaration with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set the documentation of this assignment declaration.
    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }

    /// Attempts to reverse the assignment. This is (currently) only possible for object assignments
    pub fn reverse(&self) -> Result<AssignDeclaration> {
        match self.assignment().kind() {
            AssignmentKind::Object(object) => Ok(object.object().assign(
                &Assignment::from(
                    ObjectAssignment::from(self.object().clone())
                        .assign_from(self.assignment().to_field())?,
                )
                .to_nested(object.from_field()),
            )?),
            AssignmentKind::Direct(_) => Err(Error::InvalidTarget(
                "Cannot reverse a direct assignment.".to_string(),
            )),
        }
    }
}

impl Document for AssignDeclaration {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

/// An object can be assigned from another object or directly
#[derive(Debug, Clone)]
pub struct Assignment {
    /// Indicates assignment to (nested) fields. (Named or range)
    to_field: Vec<FieldSelection>,
    /// Indicates the kind of assignment (object to object, or directly to object)
    kind: AssignmentKind,
}

impl Assignment {
    pub fn to(mut self, to: FieldSelection) -> Self {
        self.to_field.push(to);
        self
    }

    pub fn to_nested(mut self, nested: &Vec<FieldSelection>) -> Self {
        self.to_field.extend(nested.clone());
        self
    }

    /// Append a named field selection
    pub fn to_named(self, to: &str) -> Self {
        self.to(FieldSelection::Name(to.to_string()))
    }

    /// Append a range field selection
    pub fn to_range(self, to: RangeConstraint) -> Self {
        self.to(FieldSelection::Range(to))
    }

    /// Append a downto range field selection
    pub fn to_downto(self, start: i32, end: i32) -> Result<Self> {
        Ok(self.to_range(RangeConstraint::downto(start, end)?))
    }

    /// Append a to range field selection
    pub fn to_to(self, start: i32, end: i32) -> Result<Self> {
        Ok(self.to_range(RangeConstraint::to(start, end)?))
    }

    /// Append a to range field selection
    pub fn to_index(self, index: i32) -> Self {
        self.to_range(RangeConstraint::Index(index))
    }

    /// Returns the fields selected
    pub fn to_field(&self) -> &Vec<FieldSelection> {
        &self.to_field
    }

    /// Returns the assignment kind
    pub fn kind(&self) -> &AssignmentKind {
        &self.kind
    }

    pub fn declare_for(&self, object_identifier: String, pre: &str, post: &str) -> Result<String> {
        if let AssignmentKind::Direct(DirectAssignment::Value(ValueAssignment::BitVec(bitvec))) =
            self.kind()
        {
            if let Some(FieldSelection::Range(range)) = self.to_field().last() {
                return bitvec.declare_for_range(range);
            }
        }
        self.kind().declare_for(object_identifier, pre, post)
    }
}

/// An object can be assigned a value or from another object
#[derive(Debug, Clone)]
pub enum AssignmentKind {
    /// An object is assigned from or driven by another object
    Object(ObjectAssignment),
    /// An object is assigned a value, or all fields are assigned/driven at once
    Direct(DirectAssignment),
}

impl AssignmentKind {
    pub fn full_record(fields: IndexMap<String, AssignmentKind>) -> AssignmentKind {
        AssignmentKind::Direct(DirectAssignment::FullRecord(fields))
    }

    /// Converts an object assignment into a direct assignment. Useful when array or record types have identical fields but different type names.
    ///
    /// `convert_all` will also unwrap further nested objects
    pub fn to_direct(
        object: &(impl Into<ObjectAssignment> + Clone),
        convert_all: bool,
    ) -> Result<AssignmentKind> {
        let object = object.clone().into();
        match object.typ()? {
            ObjectType::Bit => Ok(object.into()),
            ObjectType::Record(rec) => {
                let mut fields = IndexMap::new();
                for (field, typ) in rec.fields() {
                    match typ {
                        ObjectType::Array(_) if convert_all => {
                            fields.insert(
                                field.clone(),
                                AssignmentKind::to_direct(
                                    &object
                                        .clone()
                                        .assign_from(&vec![FieldSelection::name(field)])?,
                                    true,
                                )?,
                            );
                        }
                        ObjectType::Record(_) if convert_all => {
                            fields.insert(
                                field.clone(),
                                AssignmentKind::to_direct(
                                    &object
                                        .clone()
                                        .assign_from(&vec![FieldSelection::name(field)])?,
                                    true,
                                )?,
                            );
                        }
                        _ => {
                            fields.insert(
                                field.clone(),
                                object
                                    .clone()
                                    .assign_from(&vec![FieldSelection::name(field)])?
                                    .into(),
                            );
                        }
                    }
                }
                Ok(AssignmentKind::Direct(DirectAssignment::FullRecord(fields)))
            }
            ObjectType::Array(arr) => {
                if arr.is_bitvector() {
                    Ok(object.into())
                } else {
                    let mut fields = vec![];
                    match arr.typ() {
                        ObjectType::Array(_) if convert_all => {
                            for i in arr.low()..arr.high() + 1 {
                                fields.push(AssignmentKind::to_direct(
                                    &object
                                        .clone()
                                        .assign_from(&vec![FieldSelection::index(i)])?,
                                    true,
                                )?);
                            }
                        }
                        ObjectType::Record(_) if convert_all => {
                            for i in arr.low()..arr.high() + 1 {
                                fields.push(AssignmentKind::to_direct(
                                    &object
                                        .clone()
                                        .assign_from(&vec![FieldSelection::index(i)])?,
                                    true,
                                )?);
                            }
                        }
                        _ => {
                            for i in arr.low()..arr.high() + 1 {
                                fields.push(
                                    object
                                        .clone()
                                        .assign_from(&vec![FieldSelection::index(i)])?
                                        .into(),
                                );
                            }
                        }
                    }
                    Ok(AssignmentKind::Direct(DirectAssignment::FullArray(
                        ArrayAssignment::Direct(fields),
                    )))
                }
            }
        }
    }

    pub fn declare_for(
        &self,
        object_identifier: impl Into<String>,
        pre: &str,
        post: &str,
    ) -> Result<String> {
        let object_identifier: &str = &object_identifier.into();
        match self {
            AssignmentKind::Object(object) => Ok(object.to_string()),
            AssignmentKind::Direct(direct) => match direct {
                DirectAssignment::Value(value) => match value {
                    ValueAssignment::Bit(bit) => Ok(format!("'{}'", bit)),
                    ValueAssignment::BitVec(bitvec) => Ok(bitvec.declare_for(object_identifier)),
                },
                DirectAssignment::FullRecord(record) => {
                    let mut field_assignments = Vec::new();
                    let nested_pre = &format!("{}  ", pre);
                    for (field, value) in record {
                        field_assignments.push(format!(
                            "\n{}{} => {}",
                            nested_pre,
                            field,
                            value.declare_for(
                                format!("{}.{}", object_identifier, field),
                                nested_pre,
                                post
                            )?
                        ));
                    }
                    Ok(format!("({}\n{})", field_assignments.join(","), pre))
                }
                DirectAssignment::FullArray(array) => match array {
                    ArrayAssignment::Direct(direct) => {
                        let mut positionals = Vec::new();
                        for value in direct {
                            positionals.push(value.declare_for(
                                format!("{}'element", object_identifier),
                                pre,
                                post,
                            )?);
                        }
                        Ok(format!("( {} )", positionals.join(", ")))
                    }
                    ArrayAssignment::Sliced { direct, others } => {
                        let mut field_assignments = Vec::new();
                        let nested_pre = &format!("{}  ", pre);
                        for (range, value) in direct {
                            field_assignments.push(format!(
                                "\n{}{} => {}",
                                nested_pre,
                                range.to_string().replace("(", "").replace(")", ""),
                                value.declare_for(
                                    format!("{}'element", object_identifier),
                                    nested_pre,
                                    post
                                )?
                            ));
                        }
                        if let Some(value) = others {
                            field_assignments.push(format!(
                                "\n{}others => {}",
                                nested_pre,
                                value.declare_for(
                                    format!("{}'element", object_identifier),
                                    nested_pre,
                                    post
                                )?
                            ));
                        }
                        Ok(format!("({}\n{})", field_assignments.join(","), pre))
                    }
                    ArrayAssignment::Others(value) => Ok(format!(
                        "( others => {} )",
                        value.declare_for(format!("{}'element", object_identifier), pre, post)?
                    )),
                },
            },
        }
    }
}

/// An object can be assigned a value or another object
#[derive(Debug, Clone)]
pub struct ObjectAssignment {
    /// The object being assigned from
    object: Box<ObjectDeclaration>,
    /// Optional selections on the object being assigned from, representing nested selections
    from_field: Vec<FieldSelection>,
}

impl fmt::Display for ObjectAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = self.object().identifier().to_string();
        for field in self.from_field() {
            result.push_str(&field.to_string());
        }
        write!(f, "{}", result)
    }
}

impl ObjectAssignment {
    /// Returns a reference to the object being assigned from
    pub fn object(&self) -> &ObjectDeclaration {
        &self.object
    }

    /// Select fields from the object being assigned
    pub fn assign_from(mut self, fields: &Vec<FieldSelection>) -> Result<Self> {
        let mut object = self.object().typ().clone();
        // Verify the fields exist
        for field in self.from_field() {
            object = object.get_field(field)?;
        }
        for field in fields {
            object = object.get_field(field)?;
            self.from_field.push(field.clone())
        }

        Ok(self)
    }

    pub fn from_field(&self) -> &Vec<FieldSelection> {
        &self.from_field
    }

    /// Returns the object type of the selected field
    pub fn typ(&self) -> Result<ObjectType> {
        let mut object = self.object().typ().clone();
        for field in self.from_field() {
            object = object.get_field(field)?;
        }
        Ok(object)
    }
}

/// Possible values which can be assigned to std_logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Directly assigning a value or an entire Record/Array
#[derive(Debug, Clone)]
pub enum DirectAssignment {
    /// Assigning a specific value to a bit vector or single bit
    Value(ValueAssignment),
    /// Assigning all fields of a Record
    FullRecord(IndexMap<String, AssignmentKind>),
    /// Assigning all fields of an Array
    FullArray(ArrayAssignment),
}

/// Directly assigning a value or an entire Record, corresponds to the Types defined in `tydi::generator::common::Type`
#[derive(Debug, Clone)]
pub enum ValueAssignment {
    /// Assigning a value to a single bit
    Bit(StdLogicValue),
    /// Assigning a value to a (part of) a bit vector
    BitVec(BitVecValue),
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
    pub fn to(start: impl Into<i32>, end: impl Into<i32>) -> Result<FieldSelection> {
        Ok(FieldSelection::Range(RangeConstraint::to(
            start.into(),
            end.into(),
        )?))
    }

    pub fn downto(start: impl Into<i32>, end: impl Into<i32>) -> Result<FieldSelection> {
        Ok(FieldSelection::Range(RangeConstraint::downto(
            start.into(),
            end.into(),
        )?))
    }

    pub fn index(index: impl Into<i32>) -> FieldSelection {
        FieldSelection::Range(RangeConstraint::Index(index.into()))
    }

    pub fn name(name: impl Into<String>) -> FieldSelection {
        FieldSelection::Name(name.into())
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

    /// Verifies whether a range constraint overlaps with this range constraint
    pub fn overlaps(&self, other: &RangeConstraint) -> bool {
        self.low() <= other.high() && other.low() <= self.high()
    }

    /// Verifies whether a range constraint is inside of this range constraint
    pub fn contains(&self, other: &RangeConstraint) -> bool {
        self.high() >= other.high() && self.low() <= other.low()
    }

    /// Verifies whether this range constraint is between `high` and `low`
    pub fn is_between(&self, high: i32, low: i32) -> Result<bool> {
        if low > high {
            Err(Error::InvalidArgument(format!(
                "{} > {}! Low cannot be greater than high",
                low, high
            )))
        } else {
            Ok(high >= self.high() && low <= self.low())
        }
    }
}
