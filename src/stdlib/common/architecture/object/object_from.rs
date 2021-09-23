use super::{ArrayObject, ObjectType, RecordObject};

impl From<ArrayObject> for ObjectType {
    fn from(array: ArrayObject) -> Self {
        ObjectType::Array(array)
    }
}

impl From<RecordObject> for ObjectType {
    fn from(rec: RecordObject) -> Self {
        ObjectType::Record(rec)
    }
}
