use indexmap::IndexMap;

use crate::{
    stdlib::common::architecture::assignment::{AssignmentKind, RangeConstraint},
    Error, Result,
};

/// An enum for describing an assignment to an array
#[derive(Debug, Clone)]
pub enum ArrayAssignment {
    /// Assigning all of an array directly (may concatenate objects)
    Direct(Vec<AssignmentKind>),
    /// Assign some fields directly, and assign all other fields a single value (e.g. ( 1 => '1', others => '0' ), or ( 1 downto 0 => '1', others => '0' ))
    Partial {
        direct: IndexMap<RangeConstraint, AssignmentKind>,
        others: Box<AssignmentKind>,
    },
    /// Assigning a single value to all of an array
    Others(Box<AssignmentKind>),
}

impl ArrayAssignment {
    pub fn direct(values: Vec<AssignmentKind>) -> ArrayAssignment {
        ArrayAssignment::Direct(values)
    }

    pub fn partial(
        direct: IndexMap<RangeConstraint, AssignmentKind>,
        others: AssignmentKind,
    ) -> ArrayAssignment {
        ArrayAssignment::Partial {
            direct,
            others: Box::new(others),
        }
    }

    pub fn others(value: AssignmentKind) -> ArrayAssignment {
        ArrayAssignment::Others(Box::new(value))
    }
}
