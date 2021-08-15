use indexmap::IndexMap;

use crate::{Error, Name, Result};

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
