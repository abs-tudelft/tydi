use super::{
    array_assignment::ArrayAssignment, bitvec::BitVecAssignment,
    record_assignment::RecordAssignment, Assignment, DirectAssignment, StdLogicValue,
    ValueAssignment,
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
