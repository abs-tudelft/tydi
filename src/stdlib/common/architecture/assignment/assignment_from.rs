use super::{
    array_assignment::ArrayAssignment, bitvec::BitVecAssignment, Assignment, DirectAssignment,
    StdLogicValue,
};

impl From<StdLogicValue> for Assignment {
    fn from(assignment: StdLogicValue) -> Self {
        Assignment::Direct(DirectAssignment::Bit(assignment))
    }
}

impl From<BitVecAssignment> for Assignment {
    fn from(assignment: BitVecAssignment) -> Self {
        Assignment::Direct(DirectAssignment::BitVec(assignment))
    }
}

impl From<ArrayAssignment> for Assignment {
    fn from(assignment: ArrayAssignment) -> Self {
        Assignment::Direct(DirectAssignment::Array(assignment))
    }
}
