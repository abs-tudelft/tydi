use crate::{stdlib::common::architecture::declaration::ObjectKind, Result};

use super::AssignedObject;

pub trait DeclareAssignment {
    /// Declare the full assignment, pre is useful for tabs/spaces, post is useful for closing characters (','/';')
    fn declare(&self, pre: &str, post: &str) -> Result<String>;
}

impl DeclareAssignment for AssignedObject {
    fn declare(&self, pre: &str, post: &str) -> Result<String> {
        let mut result = pre.to_string();
        result.push_str(&self.object_string());
        result.push_str(match self.object.kind() {
            ObjectKind::Signal => " <= ",
            ObjectKind::Variable => " := ",
            ObjectKind::Constant => " := ",
            ObjectKind::EntityPort => " <= ",
            ObjectKind::ComponentPort => " => ",
        });
        result.push_str(
            &self
                .assignment()
                .declare_for(self.object_string())?
                .replace("##pre##", pre),
        );
        result.push_str(post);
        result.push_str("\n");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use indexmap::IndexMap;

    use crate::generator::common::test::records;
    use crate::stdlib::common::architecture::assignment::{
        Assign, Assignment, AssignmentKind, StdLogicValue,
    };
    use crate::stdlib::common::architecture::declaration::{ObjectDeclaration, ObjectMode};
    use crate::stdlib::common::architecture::object::ObjectType;
    use crate::stdlib::common::architecture::{
        assignment::bitvec::BitVecValue, declaration::tests::test_complex_signal,
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

    pub(crate) fn test_record_var(
        typename: String,
        identifier: String,
    ) -> Result<ObjectDeclaration> {
        let rec_type = records::rec(typename);
        Ok(ObjectDeclaration::signal(
            identifier,
            rec_type.try_into()?,
            None,
        ))
    }

    #[test]
    fn print_bit_assign() -> Result<()> {
        let sig = test_bit_signal_object()?.assign(&StdLogicValue::Logic(false).into())?;
        let var = test_bit_variable_object()?.assign(&StdLogicValue::Logic(true).into())?;
        let port = test_bit_component_port_object()?.assign(&StdLogicValue::DontCare.into())?;
        print!("{}", sig.declare("", ";")?);
        print!("{}", var.declare("", ";")?);
        print!("{}", port.declare("   ", ",")?);
        Ok(())
    }

    #[test]
    fn print_bitvec_assign() -> Result<()> {
        let a_others = BitVecValue::Others(StdLogicValue::Logic(true));
        let a_unsigned = BitVecValue::Unsigned(32);
        let a_unsigned_range = BitVecValue::Unsigned(32);
        let a_signed = BitVecValue::Signed(-32);
        let a_signed_range = BitVecValue::Signed(-32);
        let a_str = BitVecValue::from_str("1-XUL0H")?;
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
            AssignedObject::new(
                test_complex_signal()?,
                Assignment::from(a_unsigned_range).to_downto(10, 0)?
            )
            .declare("", ";")?
        );
        print!(
            "{}",
            AssignedObject::new(test_complex_signal()?, a_signed.clone().into()).declare("", ";")?
        );
        // This won't work, because assign actually checks whether it's possible to assign this :)
        // print!(
        //     "{}",
        //     test_complex_signal()?.assign(&a_signed.clone().into())?.declare("", ";")?
        // );
        // But this will
        print!(
            "{}",
            test_complex_signal()?
                .assign(&Assignment::from(a_signed.clone()).to_named("a").to_downto(4, -3)?)?
                .declare("", ";")?
        );
        print!(
            "{}",
            AssignedObject::new(
                test_complex_signal()?,
                Assignment::from(a_signed_range).to_to(0, 10)?
            )
            .declare("", ";")?
        );
        print!(
            "{}",
            AssignedObject::new(test_complex_signal()?, a_str.into()).declare("", ";")?
        );
        Ok(())
    }

    #[test]
    fn print_record_assign() -> Result<()> {
        let a_single = BitVecValue::Others(StdLogicValue::H);
        let mut multifields = IndexMap::new();
        multifields.insert(
            "c".to_string(),
            BitVecValue::Others(StdLogicValue::H).into(),
        );
        multifields.insert("d".to_string(), BitVecValue::Signed(-55).into());
        let a_full = AssignmentKind::full_record(multifields);
        print!(
            "{}",
            AssignedObject::new(
                test_record_var("rectype".to_string(), "recname".to_string())?,
                Assignment::from(a_single.clone()).to_named("c")
            )
            .declare("", ";")?
        );
        print!(
            "{}",
            AssignedObject::new(
                test_record_var("rectype".to_string(), "recname2".to_string())?,
                Assignment::from(a_single.clone())
                    .to_named("c")
                    .to_downto(40, 30)?
            )
            .declare("", ";")?
        );
        print!(
            "{}",
            AssignedObject::new(
                test_record_var("rectype".to_string(), "recname3".to_string())?,
                a_full.into()
            )
            .declare("  ", ";")?
        );
        Ok(())
    }
}
