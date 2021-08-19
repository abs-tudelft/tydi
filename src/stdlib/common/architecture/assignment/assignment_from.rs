use super::{
    array_assignment::ArrayAssignment, bitvec::BitVecAssignment,
    record_assignment::RecordAssignment, Assignment, DirectAssignment, DirectObjectAssignment,
    FieldAssignment, StdLogicValue, ValueAssignment,
};

impl From<StdLogicValue> for Assignment {
    fn from(assignment: StdLogicValue) -> Self {
        Assignment::Direct(DirectAssignment::Value(ValueAssignment::Bit(assignment)))
    }
}

impl From<BitVecAssignment> for Assignment {
    fn from(assignment: BitVecAssignment) -> Self {
        Assignment::Direct(DirectAssignment::Value(ValueAssignment::BitVec(assignment)))
    }
}

impl From<ArrayAssignment> for Assignment {
    fn from(assignment: ArrayAssignment) -> Self {
        Assignment::Direct(DirectAssignment::Array(assignment))
    }
}

impl From<RecordAssignment> for Assignment {
    fn from(assignment: RecordAssignment) -> Self {
        Assignment::Direct(DirectAssignment::Record(assignment))
    }
}

impl From<ValueAssignment> for FieldAssignment {
    fn from(assignment: ValueAssignment) -> Self {
        FieldAssignment::Value(assignment)
    }
}

impl From<DirectObjectAssignment> for FieldAssignment {
    fn from(assignment: DirectObjectAssignment) -> Self {
        FieldAssignment::Object(assignment)
    }
}

impl From<StdLogicValue> for ValueAssignment {
    fn from(assignment: StdLogicValue) -> Self {
        ValueAssignment::Bit(assignment)
    }
}

impl From<BitVecAssignment> for ValueAssignment {
    fn from(assignment: BitVecAssignment) -> Self {
        ValueAssignment::BitVec(assignment)
    }
}

impl From<StdLogicValue> for FieldAssignment {
    fn from(assignment: StdLogicValue) -> Self {
        ValueAssignment::from(assignment).into()
    }
}

impl From<BitVecAssignment> for FieldAssignment {
    fn from(assignment: BitVecAssignment) -> Self {
        ValueAssignment::from(assignment).into()
    }
}
