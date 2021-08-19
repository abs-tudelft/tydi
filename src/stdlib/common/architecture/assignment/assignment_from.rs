use crate::stdlib::common::architecture::declaration::ObjectDeclaration;

use super::{
    array_assignment::ArrayAssignment, bitvec::BitVecValue, Assignment, AssignmentKind,
    DirectAssignment, ObjectAssignment, StdLogicValue, ValueAssignment,
};

// NOTE: I feel like there should be some way for Rust to recognize these connections automatically but ¯\_(ツ)_/¯

impl From<AssignmentKind> for Assignment {
    fn from(kind: AssignmentKind) -> Self {
        Assignment {
            kind,
            to_field: vec![],
        }
    }
}

impl From<ObjectAssignment> for Assignment {
    fn from(assignment: ObjectAssignment) -> Self {
        AssignmentKind::from(assignment).into()
    }
}

impl From<ObjectAssignment> for AssignmentKind {
    fn from(assignment: ObjectAssignment) -> Self {
        AssignmentKind::Object(assignment)
    }
}

impl From<ObjectDeclaration> for Assignment {
    fn from(assignment: ObjectDeclaration) -> Self {
        AssignmentKind::from(assignment).into()
    }
}

impl From<ObjectDeclaration> for AssignmentKind {
    fn from(assignment: ObjectDeclaration) -> Self {
        ObjectAssignment::from(assignment).into()
    }
}

impl From<ObjectDeclaration> for ObjectAssignment {
    fn from(object: ObjectDeclaration) -> Self {
        ObjectAssignment {
            object: Box::new(object),
            from_field: vec![],
        }
    }
}

impl From<DirectAssignment> for Assignment {
    fn from(assignment: DirectAssignment) -> Self {
        AssignmentKind::from(assignment).into()
    }
}

impl From<DirectAssignment> for AssignmentKind {
    fn from(assignment: DirectAssignment) -> Self {
        AssignmentKind::Direct(assignment)
    }
}

impl From<ValueAssignment> for Assignment {
    fn from(assignment: ValueAssignment) -> Self {
        AssignmentKind::from(assignment).into()
    }
}

impl From<ValueAssignment> for AssignmentKind {
    fn from(assignment: ValueAssignment) -> Self {
        DirectAssignment::from(assignment).into()
    }
}

impl From<ValueAssignment> for DirectAssignment {
    fn from(assignment: ValueAssignment) -> Self {
        DirectAssignment::Value(assignment)
    }
}

impl From<StdLogicValue> for Assignment {
    fn from(assignment: StdLogicValue) -> Self {
        AssignmentKind::from(assignment).into()
    }
}

impl From<StdLogicValue> for AssignmentKind {
    fn from(assignment: StdLogicValue) -> Self {
        ValueAssignment::from(assignment).into()
    }
}

impl From<StdLogicValue> for ValueAssignment {
    fn from(assignment: StdLogicValue) -> Self {
        ValueAssignment::Bit(assignment)
    }
}

impl From<BitVecValue> for Assignment {
    fn from(assignment: BitVecValue) -> Self {
        AssignmentKind::from(assignment).into()
    }
}

impl From<BitVecValue> for AssignmentKind {
    fn from(assignment: BitVecValue) -> Self {
        ValueAssignment::from(assignment).into()
    }
}

impl From<BitVecValue> for ValueAssignment {
    fn from(assignment: BitVecValue) -> Self {
        ValueAssignment::BitVec(assignment)
    }
}

impl From<ArrayAssignment> for Assignment {
    fn from(assignment: ArrayAssignment) -> Self {
        AssignmentKind::from(assignment).into()
    }
}

impl From<ArrayAssignment> for AssignmentKind {
    fn from(assignment: ArrayAssignment) -> Self {
        DirectAssignment::from(assignment).into()
    }
}

impl From<ArrayAssignment> for DirectAssignment {
    fn from(assignment: ArrayAssignment) -> Self {
        DirectAssignment::FullArray(assignment)
    }
}
