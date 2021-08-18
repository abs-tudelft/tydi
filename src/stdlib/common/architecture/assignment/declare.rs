use crate::{
    generator::vhdl::Declare,
    stdlib::common::architecture::{
        assignment::record_assignment::RecordAssignment,
        declaration::{ObjectDeclaration, ObjectKind},
        object::ObjectType,
    },
    Error, Result,
};

use super::{AssignedObject, Assignment, DirectAssignment};

pub trait DeclareAssignment {
    /// Declare the full assignment, pre is useful for tabs/spaces, post is useful for closing characters (','/';')
    fn declare(&self, pre: &str, post: &str) -> Result<String>;
}

impl DeclareAssignment for AssignedObject {
    fn declare(&self, pre: &str, post: &str) -> Result<String> {
        let mut result = pre.to_string();
        result.push_str(&self.object_string());
        let assign_symbol = match self.object.kind() {
            ObjectKind::Signal => " <= ",
            ObjectKind::Variable => " := ",
            ObjectKind::Constant => " := ",
            ObjectKind::EntityPort => " <= ",
            ObjectKind::ComponentPort => " => ",
        };
        match self.assignment() {
            Assignment::Object(object) => {
                result.push_str(assign_symbol);
                result.push_str(object.object().identifier());
                for field in object.from_field() {
                    result.push_str(field.to_string().as_str())
                }
            }
            Assignment::Direct(direct) => match direct {
                DirectAssignment::Value(value_assignment) => {
                    result.push_str(assign_symbol);
                    result.push_str(&value_assignment.declare_for(self.object_string()))
                }
                DirectAssignment::Record(record) => {
                    if let ObjectType::Record(record_obj) = self.object().typ() {
                        match record {
                            RecordAssignment::Single {
                                field: _,
                                assignment,
                            } => {
                                result.push_str(assign_symbol);
                                result.push_str(&assignment.declare_for(self.object_string()))
                            }
                            RecordAssignment::Multiple(assignments) => {
                                result = String::new();
                                for (key, assignment) in assignments {
                                    let obj_w_field = &format!("{}.{}", self.object_string(), key);
                                    result.push_str(&format!(
                                        "{}{}{}{}{}\n",
                                        pre,
                                        obj_w_field,
                                        assign_symbol,
                                        assignment.declare_for(obj_w_field.to_string()),
                                        post
                                    ));
                                }
                            }
                            RecordAssignment::Full(assignments) => todo!(),
                        }
                    } else {
                        return Err(Error::InvalidTarget(format!(
                            "Cannot assign Record to type {}",
                            self.object().typ()
                        )));
                    }
                }
                DirectAssignment::Union {
                    variant: _,
                    assignment: _,
                } => todo!(),
                DirectAssignment::Array(_) => todo!(),
            },
        }
        result.push_str("\n");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::stdlib::common::architecture::assignment::{RangeConstraint, StdLogicValue};
    use crate::stdlib::common::architecture::declaration::ObjectMode;
    use crate::stdlib::common::architecture::object::ObjectType;
    use crate::stdlib::common::architecture::{
        assignment::bitvec::BitVecAssignment, declaration::tests::test_complex_signal,
    };
    use crate::Result;

    use super::*;

    pub(crate) fn test_bit_signal_object() -> Result<ObjectDeclaration> {
        Ok(ObjectDeclaration::signal(
            "test_signal".to_string(),
            ObjectType::Bit,
            None,
        ))
    }

    pub(crate) fn test_bit_variable_object() -> Result<ObjectDeclaration> {
        Ok(ObjectDeclaration::variable(
            "test_variable".to_string(),
            ObjectType::Bit,
            None,
        ))
    }

    pub(crate) fn test_bit_component_port_object() -> Result<ObjectDeclaration> {
        Ok(ObjectDeclaration::component_port(
            "test_component_port".to_string(),
            ObjectType::Bit,
            ObjectMode::In,
            None,
        ))
    }

    #[test]
    fn print_bit_assign() -> Result<()> {
        let sig = AssignedObject::new(test_bit_signal_object()?, StdLogicValue::Logic(false).into());
        let var = AssignedObject::new(test_bit_variable_object()?, StdLogicValue::Logic(true).into());
        let port = AssignedObject::new(test_bit_component_port_object()?, StdLogicValue::DontCare.into());
        print!("{}", sig.declare("", ";")?);
        print!("{}", var.declare("", ";")?);
        print!("{}", port.declare("   ", ",")?);
        Ok(())
    }

    #[test]
    fn print_bitvec_assign() -> Result<()> {
        let a_others = BitVecAssignment::others(StdLogicValue::Logic(true), None)?;
        let a_unsigned = BitVecAssignment::unsigned(32, None)?;
        let a_unsigned_range =
            BitVecAssignment::unsigned(32, Some(RangeConstraint::downto(10, 0)?))?;
        let a_signed = BitVecAssignment::signed(-32, None)?;
        let a_signed_range = BitVecAssignment::signed(-32, Some(RangeConstraint::to(0, 10)?))?;
        let a_str = BitVecAssignment::from_str("1-XUL0H")?;
        print!(
            "{}",
            AssignedObject::new(test_complex_signal()?, a_others.into()).declare("", ";")?
        );
        print!(
            "{}",
            AssignedObject::new(test_complex_signal()?, a_unsigned.into()).declare("", ";")?
        );
        print!(
            "{}",
            AssignedObject::new(test_complex_signal()?, a_unsigned_range.into())
                .declare("", ";")?
        );
        print!(
            "{}",
            AssignedObject::new(test_complex_signal()?, a_signed.into()).declare("", ";")?
        );
        print!(
            "{}",
            AssignedObject::new(test_complex_signal()?, a_signed_range.into()).declare("", ";")?
        );
        print!(
            "{}",
            AssignedObject::new(test_complex_signal()?, a_str.into()).declare("", ";")?
        );
        Ok(())
    }
}
