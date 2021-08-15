use indexmap::IndexMap;

use crate::{Error, Name, Result};

use super::assignment::{FieldSelection, RangeConstraint};

/// Types of VHDL objects, possibly referring to fields
#[derive(Debug, Clone)]
pub enum ObjectType {
    /// A bit object, can not contain further fields
    Bit,
    /// An array of fields, covers both conventional arrays, as well as bit vectors
    Array(ArrayObject),
    /// A record object, consisting of named fields
    Record(IndexMap<Name, ObjectType>),
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
}
