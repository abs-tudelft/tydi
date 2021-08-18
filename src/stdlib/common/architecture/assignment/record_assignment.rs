use indexmap::IndexMap;

use super::FieldAssignment;

/// Assinging a record type
#[derive(Debug, Clone)]
pub enum RecordAssignment {
    /// Assigning a single field
    Single {
        field: String,
        assignment: Box<FieldAssignment>,
    },
    /// Assigning multiple fields
    Multiple(IndexMap<String, FieldAssignment>),
    /// Assigning all fields
    Full(IndexMap<String, FieldAssignment>),
}

impl RecordAssignment {
    pub fn single(field: String, assignment: FieldAssignment) -> RecordAssignment {
        RecordAssignment::Single {
            field,
            assignment: Box::new(assignment),
        }
    }

    pub fn multiple(fields: IndexMap<String, FieldAssignment>) -> RecordAssignment {
        RecordAssignment::Multiple(fields)
    }

    pub fn full(fields: IndexMap<String, FieldAssignment>) -> RecordAssignment {
        RecordAssignment::Full(fields)
    }
}
