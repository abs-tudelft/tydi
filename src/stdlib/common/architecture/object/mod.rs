use std::{
    convert::{TryFrom, TryInto},
    fmt,
    ops::Index,
};

use indexmap::IndexMap;

use crate::{
    generator::{
        common::{Array, Record, Type},
        vhdl::VHDLIdentifier,
    },
    stdlib::common::architecture::assignment::DirectAssignment,
    Error, Identify, Name, Result,
};

use super::assignment::{Assignment, FieldSelection, RangeConstraint};

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
                    "Record (type name: {}) with fields: ( {})",
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
                        if range.high() <= array.high() && range.low() >= array.low() {
                            // NOTE: Not sure why/if this is possible (returning a reference owned by a function)
                            Ok(ObjectType::array(
                                range.high(),
                                range.low(),
                                array.typ().clone(),
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
                FieldSelection::Range(range) => Err(Error::InvalidTarget(
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

    /// Create an array of a specific field type
    pub fn array(high: i32, low: i32, object: ObjectType) -> Result<ObjectType> {
        Ok(ObjectType::Array(ArrayObject::array(high, low, object)?))
    }

    /// Create a bit vector object
    pub fn bit_vector(high: i32, low: i32) -> Result<ObjectType> {
        ObjectType::array(high, low, ObjectType::Bit)
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
                            "Cannot assign record type {} directly to record type {}",
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
        match assignment {
            Assignment::Object(object) => {
                let mut from_object = object.typ().clone();
                for field in object.from_field() {
                    from_object = from_object.get_field(field)?;
                }
                let mut to_object = self.clone();
                for field in object.to_field() {
                    to_object = to_object.get_field(field)?;
                }
                to_object.can_assign_type(&from_object)
            }
            Assignment::Direct(direct) => match direct {
                DirectAssignment::Bit(_) => todo!(),
                DirectAssignment::BitVec(_) => todo!(),
                DirectAssignment::Record(_) => todo!(),
                DirectAssignment::Union(_, _) => todo!(),
                DirectAssignment::Array(_) => todo!(),
            },
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
            Type::Record(record) | Type::Union(record) => {
                Ok(ObjectType::Record(RecordObject::try_from(record)?))
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
}

impl RecordObject {
    pub fn new(type_name: String, fields: IndexMap<String, ObjectType>) -> RecordObject {
        RecordObject { type_name, fields }
    }

    pub fn type_name(&self) -> &str {
        self.type_name.as_str()
    }

    pub fn fields(&self) -> &IndexMap<String, ObjectType> {
        &self.fields
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
}

impl ArrayObject {
    /// Create a bit vector object
    pub fn bit_vector(high: i32, low: i32) -> Result<ArrayObject> {
        ArrayObject::array(high, low, ObjectType::Bit)
    }

    /// Create an array of a specific field type
    pub fn array(high: i32, low: i32, object: ObjectType) -> Result<ArrayObject> {
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
        (self.high - self.low).try_into().unwrap()
    }
}

impl TryFrom<Array> for ArrayObject {
    type Error = Error;

    fn try_from(value: Array) -> Result<Self> {
        Ok(ArrayObject::array(
            value.width().try_into().unwrap(),
            0,
            ObjectType::try_from(value.typ().clone())?,
        )?)
    }
}
