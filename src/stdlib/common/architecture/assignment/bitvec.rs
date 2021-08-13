use crate::stdlib::common::architecture::assignment::*;
use crate::Error;
use std::convert::TryFrom;

/// Quick way to get the minimum number of binary values required for an unsigned integer
fn min_length_unsigned(value: u32) -> u32 {
    if value == 0 {
        1
    } else {
        32 - value.leading_zeros()
    }
}

/// Quick way to get the minimum number of binary values required for a signed integer
fn min_length_signed(value: i32) -> u32 {
    if value == 0 {
        1
    } else if value < 0 {
        33 - value.leading_ones()
    } else {
        33 - value.leading_zeros()
    }
}

/// A struct for describing value assigned to a bit vector
#[derive(Debug, Clone)]
pub enum BitVecValue {
    /// Value assigned as (others => value)
    Others(StdLogicValue),
    /// A single value assigned to an index
    Indexed(StdLogicValue),
    /// A full, specific range of std_logic values is assigned
    ///
    /// Result (example): "01-XULH"
    Full(Vec<StdLogicValue>),
    /// A value is assigned from an unsigned integer
    ///
    /// Result: std_logic_vector(to_unsigned([value], [name]'length))
    ///
    /// Or: std_logic_vector(to_unsigned([value], [range length]))
    Unsigned(u32),
    /// A value is assigned from a signed integer
    ///
    /// Result: std_logic_vector(to_signed([value], [name]'length))
    ///
    /// Or: std_logic_vector(to_signed([value], [range length]))
    Signed(i32),
}

/// A struct for describing an assignment to a bit vector
#[derive(Debug, Clone)]
pub struct BitVecAssignment {
    /// When range_constraint is None, the entire range is assigned
    range_constraint: Option<RangeConstraint>,
    /// The values assigned to the range
    value: BitVecValue,
}

impl BitVecAssignment {
    /// Create a new index-based assignment of a bit vector
    pub fn index(value: StdLogicValue, index: i32) -> BitVecAssignment {
        BitVecAssignment {
            range_constraint: Some(RangeConstraint::Index(index)),
            value: BitVecValue::Indexed(value),
        }
    }

    /// Create a new downto-range assignment of a bit vector
    pub fn downto(
        value: Vec<StdLogicValue>,
        start: i32,
        end: i32,
    ) -> crate::Result<BitVecAssignment> {
        if usize::try_from(start - end)
            .map(|w| w == value.len())
            .unwrap_or(false)
        {
            Ok(BitVecAssignment {
                range_constraint: Some(RangeConstraint::downto(start, end)?),
                value: BitVecValue::Full(value),
            })
        } else {
            Err(Error::InvalidArgument(format!("Values do not match range")))
        }
    }

    /// Create a new to-range assignment of a bit vector
    pub fn to(value: Vec<StdLogicValue>, start: i32, end: i32) -> crate::Result<BitVecAssignment> {
        if usize::try_from(end - start)
            .map(|w| w == value.len())
            .unwrap_or(false)
        {
            Ok(BitVecAssignment {
                range_constraint: Some(RangeConstraint::to(start, end)?),
                value: BitVecValue::Full(value),
            })
        } else {
            Err(Error::InvalidArgument(format!("Values do not match range")))
        }
    }

    /// Create a new assignment of a bit vector, with all values assigned to `value`
    pub fn others(
        value: StdLogicValue,
        range_constraint: Option<RangeConstraint>,
    ) -> crate::Result<BitVecAssignment> {
        if let Some(RangeConstraint::Index(_)) = range_constraint {
            return Err(Error::InvalidTarget(
                "Cannot assign (others => '') to indexed std_logic".to_string(),
            ));
        }
        Ok(BitVecAssignment {
            range_constraint,
            value: BitVecValue::Others(value),
        })
    }

    /// Create a new assignment of a bit vector from an unsigned integer (natural)
    pub fn unsigned(
        value: u32,
        range_constraint: Option<RangeConstraint>,
    ) -> crate::Result<BitVecAssignment> {
        if let Some(constraint) = &range_constraint {
            if let RangeConstraint::Index(_) = constraint {
                return Err(Error::InvalidTarget(
                    "Cannot assign an std_logic_vector(unsigned) to indexed std_logic".to_string(),
                ));
            } else if min_length_unsigned(value) > constraint.width_u32() {
                return Err(Error::InvalidArgument(format!(
                    "Cannot assign unsigned integer {} to range with width {}",
                    value,
                    constraint.width_u32()
                )));
            }
        }
        Ok(BitVecAssignment {
            range_constraint,
            value: BitVecValue::Unsigned(value),
        })
    }

    /// Create a new assignment of a bit vector from a signed integer
    pub fn signed(
        value: i32,
        range_constraint: Option<RangeConstraint>,
    ) -> crate::Result<BitVecAssignment> {
        if let Some(constraint) = &range_constraint {
            if let RangeConstraint::Index(_) = constraint {
                return Err(Error::InvalidTarget(
                    "Cannot assign an std_logic_vector(signed) to indexed std_logic".to_string(),
                ));
            } else if min_length_signed(value) > constraint.width_u32() {
                return Err(Error::InvalidArgument(format!(
                    "Cannot assign signed integer {} to range with width {}",
                    value,
                    constraint.width_u32()
                )));
            }
        }
        Ok(BitVecAssignment {
            range_constraint,
            value: BitVecValue::Signed(value),
        })
    }

    /// Returns the range constraint of this assignment
    pub fn range_constraint(&self) -> Option<RangeConstraint> {
        self.range_constraint.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_length_signed_test() {
        assert_eq!(1, min_length_signed(0));
        assert_eq!(1, min_length_signed(-1));
        assert_eq!(2, min_length_signed(1));
        assert_eq!(32, min_length_signed(i32::MIN));
        assert_eq!(32, min_length_signed(i32::MAX));
    }

    #[test]
    fn min_length_unsigned_test() {
        assert_eq!(1, min_length_unsigned(0));
        assert_eq!(1, min_length_unsigned(1));
        assert_eq!(2, min_length_unsigned(2));
        assert_eq!(32, min_length_unsigned(u32::MAX));
    }
}
