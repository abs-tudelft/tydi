use crate::stdlib::common::architecture::declaration::ObjectDeclaration;

use super::{
    array_assignment::ArrayAssignment, bitvec::BitVecValue, Assignment, AssignmentKind,
    DirectAssignment, ObjectAssignment, StdLogicValue, ValueAssignment,
};

// I feel like there should be some way for Rust to recognize these connections automatically but unfortunately we can't just string "T: Into<...>"s together,
// due to potentially conflicting trait implementations: https://users.rust-lang.org/t/conflicting-implementations-of-trait/53055

impl<T> From<T> for Assignment
where
    T: Into<AssignmentKind>,
{
    fn from(kind: T) -> Self {
        Assignment {
            kind: kind.into(),
            to_field: vec![],
        }
    }
}

impl From<ObjectAssignment> for AssignmentKind {
    fn from(assignment: ObjectAssignment) -> Self {
        AssignmentKind::Object(assignment)
    }
}

impl From<ObjectDeclaration> for AssignmentKind {
    fn from(assignment: ObjectDeclaration) -> Self {
        AssignmentKind::Object(assignment.into())
    }
}

// As there are more Direct kinds, this one gets to use the where T: Into...
impl<T> From<T> for AssignmentKind
where
    T: Into<DirectAssignment>,
{
    fn from(assignment: T) -> Self {
        AssignmentKind::Direct(assignment.into())
    }
}

impl<T> From<T> for ObjectAssignment
where
    T: Into<ObjectDeclaration>,
{
    fn from(object: T) -> Self {
        ObjectAssignment {
            object: Box::new(object.into()),
            from_field: vec![],
        }
    }
}

impl<T> From<T> for DirectAssignment
where
    T: Into<ValueAssignment>,
{
    fn from(assignment: T) -> Self {
        DirectAssignment::Value(assignment.into())
    }
}

impl From<ArrayAssignment> for DirectAssignment {
    fn from(assignment: ArrayAssignment) -> Self {
        DirectAssignment::FullArray(assignment.into())
    }
}

impl From<StdLogicValue> for ValueAssignment {
    fn from(assignment: StdLogicValue) -> Self {
        ValueAssignment::Bit(assignment.into())
    }
}

impl From<BitVecValue> for ValueAssignment {
    fn from(assignment: BitVecValue) -> Self {
        ValueAssignment::BitVec(assignment.into())
    }
}
