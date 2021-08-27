use std::{
    convert::{TryFrom, TryInto},
    fmt,
};

use indexmap::IndexMap;

use crate::{
    generator::{
        common::{Array, Record, Type},
        vhdl::VHDLIdentifier,
    },
    stdlib::common::architecture::assignment::{
        array_assignment::ArrayAssignment, DirectAssignment, ValueAssignment,
    },
    Error, Identify, Result,
};

use super::assignment::{Assignment, AssignmentKind, FieldSelection, RangeConstraint};

pub mod object_from;

/// Types of VHDL objects, possibly referring to fields
#[derive(Debug, Clone)]
pub enum ObjectType {
    /// A bit object, can not contain further fields
    Bit,
    /// An array of fields, covers both conventional arrays, as well as bit vectors
    Array(ArrayObject),
    /// A record object, consisting of named fields
    Record(RecordObject),
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Bit => write!(f, "Bit"),
            ObjectType::Array(array) => write!(
                f,
                "Array ({} to {}) containing {}",
                array.low(),
                array.high(),
                array.typ()
            ),
            ObjectType::Record(record) => {
                let mut fields = String::new();
                for (name, typ) in record.fields() {
                    fields.push_str(format!("{}: {} ", name, typ).as_str());
                }
                write!(
                    f,
                    "{} (type name: {}) with fields: ( {})",
                    if record.is_union() { "Union" } else { "Record" },
                    record.type_name(),
                    fields
                )
            }
        }
    }
}

impl ObjectType {
    pub fn get_field(&self, field: &FieldSelection) -> Result<ObjectType> {
        match self {
            ObjectType::Bit => Err(Error::InvalidTarget(
                "Cannot select a field on a Bit".to_string(),
            )),
            ObjectType::Array(array) => match field {
                FieldSelection::Range(range) => {
                    if let RangeConstraint::Index(index) = range {
                        if *index <= array.high() && *index >= array.low() {
                            Ok(array.typ().clone())
                        } else {
                            Err(Error::InvalidArgument(format!(
                                "Cannot select index {} on array with high: {}, low: {}",
                                index,
                                array.high(),
                                array.low()
                            )))
                        }
                    } else {
                        if range.is_between(array.high(), array.low())? {
                            Ok(ObjectType::array(
                                range.high(),
                                range.low(),
                                array.typ().clone(),
                                array.type_name(), // NOTE: This is technically incorrect, as array types also declare the range
                            )?)
                        } else {
                            Err(Error::InvalidArgument(format!(
                                "Cannot select {} on array with high: {}, low: {}",
                                range,
                                array.high(),
                                array.low()
                            )))
                        }
                    }
                }
                FieldSelection::Name(_) => Err(Error::InvalidTarget(
                    "Cannot select a named field on an array".to_string(),
                )),
            },
            ObjectType::Record(record) => match field {
                FieldSelection::Range(_) => Err(Error::InvalidTarget(
                    "Cannot select a range on a record".to_string(),
                )),
                FieldSelection::Name(name) => Ok(record
                    .fields()
                    .get(name)
                    .ok_or(Error::InvalidArgument(format!(
                        "Field with name {} does not exist on record",
                        name
                    )))?
                    .clone()),
            },
        }
    }

    pub fn get_nested(&self, nested: &Vec<FieldSelection>) -> Result<ObjectType> {
        let mut result = self.clone();
        for field in nested {
            result = result.get_field(field)?;
        }
        Ok(result)
    }

    /// Create an array of a specific field type
    pub fn array(
        high: i32,
        low: i32,
        object: ObjectType,
        type_name: impl Into<String>,
    ) -> Result<ObjectType> {
        Ok(ObjectType::Array(ArrayObject::array(
            high, low, object, type_name,
        )?))
    }

    /// Create a bit vector object
    pub fn bit_vector(high: i32, low: i32) -> Result<ObjectType> {
        Ok(ArrayObject::bit_vector(high, low)?.into())
    }

    pub fn can_assign_type(&self, typ: &ObjectType) -> Result<()> {
        match self {
            ObjectType::Bit => {
                if let ObjectType::Bit = typ {
                    Ok(())
                } else {
                    Err(Error::InvalidTarget(format!(
                        "Cannot assign {} to Bit",
                        typ
                    )))
                }
            }
            ObjectType::Array(to_array) => {
                if let ObjectType::Array(from_array) = typ {
                    if from_array.width() == to_array.width() {
                        to_array.typ().can_assign_type(from_array.typ())
                    } else {
                        Err(Error::InvalidTarget(format!(
                            "Cannot assign array with width {} to array with width {}",
                            from_array.width(),
                            to_array.width(),
                        )))
                    }
                } else {
                    Err(Error::InvalidTarget(format!(
                        "Cannot assign {} to Array",
                        typ
                    )))
                }
            }
            ObjectType::Record(to_record) => {
                if let ObjectType::Record(from_record) = typ {
                    if from_record.type_name() == to_record.type_name() {
                        Ok(())
                    } else {
                        Err(Error::InvalidTarget(format!(
                            "Cannot assign record type {} to record type {}",
                            from_record.type_name(),
                            to_record.type_name(),
                        )))
                    }
                } else {
                    Err(Error::InvalidTarget(format!(
                        "Cannot assign {} to {}",
                        typ, self
                    )))
                }
            }
        }
    }

    pub fn can_assign(&self, assignment: &Assignment) -> Result<()> {
        let mut to_object = self.clone();
        for field in assignment.to_field() {
            to_object = to_object.get_field(field)?;
        }
        match assignment.kind() {
            AssignmentKind::Object(object) => to_object.can_assign_type(&object.typ()?),
            AssignmentKind::Direct(direct) => match direct {
                DirectAssignment::Value(value) => match value {
                    ValueAssignment::Bit(_) => match to_object {
                        ObjectType::Bit => Ok(()),
                        ObjectType::Array(_) | ObjectType::Record(_) => Err(Error::InvalidTarget(
                            format!("Cannot assign Bit to {}", to_object),
                        )),
                    },
                    ValueAssignment::BitVec(bitvec) => match to_object {
                        ObjectType::Array(array) if array.is_bitvector() => {
                            bitvec.validate_width(array.width())
                        }
                        _ => Err(Error::InvalidTarget(format!(
                            "Cannot assign Bit Vector to {}",
                            to_object
                        ))),
                    },
                },
                DirectAssignment::FullRecord(record) => {
                    if let ObjectType::Record(to_record) = &to_object {
                        if to_record.fields().len() == record.len() {
                            for (field, value) in record {
                                let to_field = to_object.get_field(&FieldSelection::name(field))?;
                                to_field.can_assign(&Assignment::from(value.clone()))?;
                            }
                            Ok(())
                        } else {
                            Err(Error::InvalidArgument(format!("Attempted full record assignment. Number of fields do not match. Record has {} fields, assignment has {} fields", to_record.fields().len(), record.len())))
                        }
                    } else {
                        Err(Error::InvalidTarget(format!(
                            "Cannot perform full Record assignment to {}",
                            to_object
                        )))
                    }
                }
                DirectAssignment::FullArray(array) => {
                    if let ObjectType::Array(to_array) = &to_object {
                        match array {
                            ArrayAssignment::Direct(direct) => {
                                if to_array.width() == direct.len().try_into().unwrap() {
                                    for value in direct {
                                        to_array
                                            .typ()
                                            .can_assign(&Assignment::from(value.clone()))?;
                                    }
                                    Ok(())
                                } else {
                                    Err(Error::InvalidArgument(format!("Attempted full array assignment. Number of fields do not match. Array has {} fields, assignment has {} fields", to_array.width(), direct.len())))
                                }
                            }
                            ArrayAssignment::Sliced { direct, others } => {
                                let mut ranges_assigned: Vec<&RangeConstraint> = vec![];
                                for (range, value) in direct {
                                    if !range.is_between(to_array.high(), to_array.low())? {
                                        return Err(Error::InvalidArgument(format!(
                                            "{} is not between {} and {}",
                                            range,
                                            to_array.high(),
                                            to_array.low()
                                        )));
                                    }
                                    if ranges_assigned.iter().any(|x| x.overlaps(range)) {
                                        return Err(Error::InvalidArgument(format!("Sliced array assignment: {} overlaps with a range which was already assigned.", range)));
                                    }
                                    to_array
                                        .typ()
                                        .can_assign(&Assignment::from(value.clone()))?;
                                    ranges_assigned.push(range);
                                }
                                let total_assigned: u32 =
                                    ranges_assigned.iter().map(|x| x.width_u32()).sum();
                                if total_assigned == to_array.width() {
                                    if let Some(_) = others {
                                        return Err(Error::InvalidArgument("Sliced array assignment contains an 'others' field, but already assigns all fields directly.".to_string()));
                                    } else {
                                        Ok(())
                                    }
                                } else {
                                    if let Some(value) = others {
                                        to_array
                                            .typ()
                                            .can_assign(&Assignment::from(value.as_ref().clone()))
                                    } else {
                                        Err(Error::InvalidArgument("Sliced array assignment does not assign all values directly, but does not contain an 'others' field.".to_string()))
                                    }
                                }
                            }
                            ArrayAssignment::Others(others) => to_array
                                .typ()
                                .can_assign(&Assignment::from(others.as_ref().clone())),
                        }
                    } else {
                        Err(Error::InvalidTarget(format!(
                            "Cannot perform full Array assignment to {}",
                            to_object
                        )))
                    }
                }
            },
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            ObjectType::Bit => "std_logic",
            ObjectType::Array(array) => array.type_name(),
            ObjectType::Record(record) => record.type_name(),
        }
    }

    /// Returns true if the object is a Bit or Bit Vector
    pub fn is_flat(&self) -> bool {
        match self {
            ObjectType::Bit => true,
            ObjectType::Array(arr) if arr.is_bitvector() => true,
            _ => false,
        }
    }
}

impl TryFrom<Type> for ObjectType {
    fn try_from(typ: Type) -> Result<Self> {
        match typ {
            Type::Bit => Ok(ObjectType::Bit),
            Type::BitVec { width } => {
                Ok(ObjectType::bit_vector((width - 1).try_into().unwrap(), 0)?)
            }
            Type::Record(record) => Ok(ObjectType::Record(RecordObject::try_from(record)?)),
            Type::Union(record) => {
                let mut fields = IndexMap::new();
                for field in record.fields() {
                    fields.insert(
                        field.identifier().to_string(),
                        ObjectType::try_from(field.typ().clone())?,
                    );
                }
                Ok(ObjectType::Record(RecordObject::new_union(
                    record.vhdl_identifier()?,
                    fields,
                )))
            }
            Type::Array(array) => Ok(ObjectType::Array(ArrayObject::try_from(array)?)),
        }
    }

    type Error = Error;
}

/// An record object
#[derive(Debug, Clone)]
pub struct RecordObject {
    type_name: String,
    fields: IndexMap<String, ObjectType>,
    /// While Unions are record objects, care needs to be taken to ensure their (non-tag) fields are always assigned from the same signal
    is_union: bool,
}

impl RecordObject {
    pub fn new(type_name: String, fields: IndexMap<String, ObjectType>) -> RecordObject {
        RecordObject {
            type_name,
            fields,
            is_union: false,
        }
    }

    /// While Unions are record objects, care needs to be taken to ensure their (non-tag) fields are always assigned from the same signal
    pub fn new_union(type_name: String, fields: IndexMap<String, ObjectType>) -> RecordObject {
        RecordObject {
            type_name,
            fields,
            is_union: true,
        }
    }

    pub fn type_name(&self) -> &str {
        self.type_name.as_str()
    }

    pub fn fields(&self) -> &IndexMap<String, ObjectType> {
        &self.fields
    }

    /// While Unions are record objects, care needs to be taken to ensure their (non-tag) fields are always assigned from the same signal
    pub fn is_union(&self) -> bool {
        self.is_union
    }

    pub fn get_field(&self, field_name: impl Into<String>) -> Result<&ObjectType> {
        let field_name = &field_name.into();
        self.fields()
            .get(field_name)
            .ok_or(Error::InvalidArgument(format!(
                "Field {} does not exist on record with type {}",
                field_name,
                self.type_name()
            )))
    }
}

impl TryFrom<Record> for RecordObject {
    type Error = Error;

    fn try_from(value: Record) -> Result<Self> {
        let mut fields = IndexMap::new();
        for field in value.fields() {
            fields.insert(
                field.identifier().to_string(),
                ObjectType::try_from(field.typ().clone())?,
            );
        }
        Ok(RecordObject::new(value.vhdl_identifier()?, fields))
    }
}

/// An array object, arrays contain a single type of object, but can contain nested objects
#[derive(Debug, Clone)]
pub struct ArrayObject {
    high: i32,
    low: i32,
    typ: Box<ObjectType>,
    type_name: String,
}

impl ArrayObject {
    /// Create a bit vector object
    pub fn bit_vector(high: i32, low: i32) -> Result<ArrayObject> {
        ArrayObject::array(
            high,
            low,
            ObjectType::Bit,
            format!("std_logic_vector({} downto {})", high, low),
        )
    }

    /// Create an array of a specific field type
    pub fn array(
        high: i32,
        low: i32,
        object: ObjectType,
        type_name: impl Into<String>,
    ) -> Result<ArrayObject> {
        if low > high {
            Err(Error::InvalidArgument(format!(
                "{} > {}! Low must be lower than high",
                low, high
            )))
        } else {
            Ok(ArrayObject {
                high,
                low,
                typ: Box::new(object),
                type_name: type_name.into(),
            })
        }
    }

    pub fn typ(&self) -> &ObjectType {
        &self.typ
    }

    pub fn high(&self) -> i32 {
        self.high
    }

    pub fn low(&self) -> i32 {
        self.low
    }

    pub fn width(&self) -> u32 {
        (1 + self.high - self.low).try_into().unwrap()
    }

    pub fn is_bitvector(&self) -> bool {
        match self.typ() {
            ObjectType::Bit => true,
            _ => false,
        }
    }

    pub fn type_name(&self) -> &str {
        self.type_name.as_str()
    }
}

impl TryFrom<Array> for ArrayObject {
    type Error = Error;

    fn try_from(value: Array) -> Result<Self> {
        Ok(ArrayObject::array(
            value.width().try_into().unwrap(),
            0,
            ObjectType::try_from(value.typ().clone())?,
            value.vhdl_identifier()?,
        )?)
    }
}
