use crate::{
    stdlib::common::architecture::assignment::{Assignment, RangeConstraint},
    Error, Result,
};

use super::DirectAssignment;

/// An enum for describing an assignment to an array
#[derive(Debug, Clone)]
pub enum ArrayAssignment {
    /// Assigning a range of an array with a value
    Range(ArrayRangeAssignment),
    /// Assigning all of an array directly (may concatenate objects)
    Direct(Vec<Assignment>),
    /// Assigning a single value to all of an array
    Others(Box<DirectAssignment>),
}

#[derive(Debug, Clone)]
pub struct ArrayRangeAssignment {
    assignment: Box<DirectAssignment>,
    range_constraint: RangeConstraint,
}

impl ArrayRangeAssignment {
    pub fn new(
        assignment: DirectAssignment,
        range_constraint: RangeConstraint,
    ) -> Result<ArrayRangeAssignment> {
        match assignment {
            DirectAssignment::Bit(_) => match range_constraint {
                RangeConstraint::Index(_) => Ok(ArrayRangeAssignment {
                    assignment: Box::new(assignment),
                    range_constraint,
                }),
                _ => Err(Error::InvalidTarget(format!(
                    "Cannot assign Bit to range {}",
                    range_constraint
                ))),
            },
            DirectAssignment::BitVec(_) => todo!(),
            DirectAssignment::Record(_) => todo!(),
            DirectAssignment::Union(_, _) => todo!(),
            DirectAssignment::Array(_) => todo!(),
        }
    }
}
