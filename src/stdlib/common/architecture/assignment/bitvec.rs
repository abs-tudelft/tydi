use crate::stdlib::common::architecture::assignment::*;
use crate::Error;


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

impl BitVecValue {
    /// Create a bit vector value from a string
    pub fn from_str(value: &str) -> Result<BitVecValue> {
        let logicvals = value
            .chars()
            .map(StdLogicValue::from_char)
            .collect::<Result<Vec<StdLogicValue>>>()?;
        Ok(BitVecValue::Full(logicvals))
    }

    pub fn validate_width(&self, width: u32) -> Result<()> {
        match self {
            BitVecValue::Others(_) => Ok(()),
            BitVecValue::Full(full) => {
                if full.len() == width.try_into().unwrap() {
                    Ok(())
                } else {
                    Err(Error::InvalidArgument(format!(
                        "Value with length {} cannot be assigned to bit vector with length {}",
                        full.len(),
                        width
                    )))
                }
            }
            BitVecValue::Unsigned(value) => {
                if min_length_unsigned(*value) > width {
                    Err(Error::InvalidArgument(format!(
                        "Cannot assign unsigned integer {} to range with width {}",
                        value, width
                    )))
                } else {
                    Ok(())
                }
            }
            BitVecValue::Signed(value) => {
                if min_length_signed(*value) > width {
                    Err(Error::InvalidArgument(format!(
                        "Cannot assign signed integer {} to range with width {}",
                        value, width
                    )))
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn declare(&self) -> Result<String> {
        match self {
            BitVecValue::Others(value) => Ok(format!("(others => '{}')", value)),
            BitVecValue::Full(bitvec) => {
                let mut result = String::new();
                for value in bitvec {
                    result.push_str(value.to_string().as_str());
                }
                Ok(format!("\"{}\"", result))
            }
            BitVecValue::Unsigned(_) | BitVecValue::Signed(_) => Err(Error::InvalidTarget("Unable to declare bit vector value, signed and unsigned values require a width or object identifier.".to_string())),
        }
    }

    /// Declares the value assigned for the object being assigned to (identifier required in case Range is empty)
    pub fn declare_for(&self, object_identifier: impl Into<String>) -> String {
        match self {
            BitVecValue::Others(_) | BitVecValue::Full(_) => self.declare().unwrap(),
            BitVecValue::Unsigned(value) => format!(
                "std_logic_vector(to_unsigned({}, {}'length))",
                value,
                object_identifier.into()
            ),
            BitVecValue::Signed(value) => format!(
                "std_logic_vector(to_signed({}, {}'length))",
                value,
                object_identifier.into()
            ),
        }
    }

    /// Declares the value assigned for the range being assigned to
    pub fn declare_for_range(&self, range: &RangeConstraint) -> Result<String> {
        match self {
            BitVecValue::Others(_) | BitVecValue::Full(_) => self.declare(),
            BitVecValue::Unsigned(value) => match range.width() {
                Width::Scalar => Err(Error::InvalidTarget(
                    "Cannot assign an std_logic_vector(unsigned) to indexed std_logic".to_string(),
                )),
                Width::Vector(width) => {
                    self.validate_width(width)?;
                    Ok(format!(
                        "std_logic_vector(to_unsigned({}, {}))",
                        value, width
                    ))
                }
            },
            BitVecValue::Signed(value) => match range.width() {
                Width::Scalar => Err(Error::InvalidTarget(
                    "Cannot assign an std_logic_vector(signed) to indexed std_logic".to_string(),
                )),
                Width::Vector(width) => {
                    self.validate_width(width)?;
                    Ok(format!("std_logic_vector(to_signed({}, {}))", value, width))
                }
            },
        }
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
