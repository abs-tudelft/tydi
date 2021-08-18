use crate::{
    generator::vhdl::Declare,
    stdlib::common::architecture::declaration::{ObjectDeclaration, ObjectKind},
    Result,
};

use super::{AssignedObject, Assignment, DirectAssignment};

impl Declare for ObjectDeclaration {
    fn declare(&self) -> Result<String> {
        todo!()
    }
}

impl Declare for AssignedObject {
    fn declare(&self) -> Result<String> {
        let mut result = self.object.identifier().to_string();
        let assign_symbol = match self.object.kind() {
            ObjectKind::Signal => " <= ",
            ObjectKind::Variable => " := ",
            ObjectKind::Constant => " := ",
            ObjectKind::EntityPort => " <= ",
            ObjectKind::ComponentPort => " => ",
        };
        match self.assignment() {
            Assignment::Object(object) => {
                for field in object.to_field() {
                    result.push_str(field.to_string().as_str())
                }
                result.push_str(assign_symbol);
                result.push_str(object.object().identifier());
                for field in object.from_field() {
                    result.push_str(field.to_string().as_str())
                }
            }
            Assignment::Direct(direct) => match direct {
                DirectAssignment::Bit(value) => {
                    result = format!("{}{}'{}'", result, assign_symbol, value);
                }
                DirectAssignment::BitVec(bitvec) => {
                    if let Some(range_constraint) = bitvec.range_constraint() {
                        result.push_str(&range_constraint.to_string());
                    }
                    result.push_str(assign_symbol);
                    result.push_str(
                        bitvec
                            .declare_for(self.object.identifier().to_string())
                            .as_str(),
                    )
                }
                DirectAssignment::Record(_) => todo!(),
                DirectAssignment::Union(_, _) => todo!(),
                DirectAssignment::Array(_) => todo!(),
            },
        }
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
        let assignment = Assignment::Direct(DirectAssignment::Bit(StdLogicValue::Logic(false)));
        let sig = AssignedObject::new(test_bit_signal_object()?, assignment.clone());
        let var = AssignedObject::new(test_bit_variable_object()?, assignment.clone());
        let port = AssignedObject::new(test_bit_component_port_object()?, assignment.clone());
        print!("{}\n", sig.declare()?);
        print!("{}\n", var.declare()?);
        print!("{}\n", port.declare()?);
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
            "{}\n",
            AssignedObject::new(test_complex_signal()?, a_others.into()).declare()?
        );
        print!(
            "{}\n",
            AssignedObject::new(test_complex_signal()?, a_unsigned.into()).declare()?
        );
        print!(
            "{}\n",
            AssignedObject::new(test_complex_signal()?, a_unsigned_range.into()).declare()?
        );
        print!(
            "{}\n",
            AssignedObject::new(test_complex_signal()?, a_signed.into()).declare()?
        );
        print!(
            "{}\n",
            AssignedObject::new(test_complex_signal()?, a_signed_range.into()).declare()?
        );
        print!(
            "{}\n",
            AssignedObject::new(test_complex_signal()?, a_str.into()).declare()?
        );
        Ok(())
    }
}
