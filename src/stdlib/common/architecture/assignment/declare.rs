use crate::stdlib::common::architecture::ArchitectureDeclare;
use crate::{stdlib::common::architecture::declaration::ObjectKind, Document, Result};

use super::AssignDeclaration;

impl ArchitectureDeclare for AssignDeclaration {
    fn declare(&self, pre: &str, post: &str) -> Result<String> {
        let mut result = pre.to_string();
        if let Some(doc) = self.doc() {
            result.push_str("--");
            result.push_str(doc.replace("\n", &format!("\n{}--", pre)).as_str());
            result.push_str("\n");
            result.push_str(pre);
        }
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
                .declare_for(self.object_string(), pre, post)?,
        );
        result.push_str(post);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use indexmap::IndexMap;

    use crate::generator::common::test::records;
    use crate::generator::common::Mode;
    use crate::stdlib::common::architecture::assignment::{
        Assign, Assignment, AssignmentKind, StdLogicValue,
    };
    use crate::stdlib::common::architecture::declaration::ObjectDeclaration;
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
            Mode::In,
        ))
    }

    pub(crate) fn test_record_signal(
        typename: impl Into<String>,
        identifier: impl Into<String>,
    ) -> Result<ObjectDeclaration> {
        let rec_type = records::rec(typename);
        Ok(ObjectDeclaration::signal(
            identifier,
            rec_type.try_into()?,
            None,
        ))
    }

    pub(crate) fn test_bitvec_signal(
        identifier: impl Into<String>,
        high: i32,
        low: i32,
    ) -> Result<ObjectDeclaration> {
        Ok(ObjectDeclaration::signal(
            identifier,
            ObjectType::bit_vector(high, low)?,
            None,
        ))
    }

    #[test]
    fn test_bit_assign() -> Result<()> {
        let sig = test_bit_signal_object()?.assign(&StdLogicValue::Logic(false))?;
        let var = test_bit_variable_object()?.assign(&StdLogicValue::Logic(true))?;
        let port = test_bit_component_port_object()?
            .assign(&StdLogicValue::DontCare)?
            .with_doc("This is\nSome neat documentation");
        assert_eq!(sig.declare("", ";\n")?, "test_signal <= '0';\n");
        assert_eq!(var.declare("", ";\n")?, "test_variable := '1';\n");
        assert_eq!(
            port.declare("   ", ",\n")?,
            r#"   --This is
   --Some neat documentation
   test_component_port => '-',
"#
        );
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
            AssignDeclaration::new(test_complex_signal()?, a_others.into()).declare("", ";\n")?
        );
        print!(
            "{}",
            AssignDeclaration::new(test_complex_signal()?, a_unsigned.into()).declare("", ";\n")?
        );
        print!(
            "{}",
            AssignDeclaration::new(
                test_complex_signal()?,
                Assignment::from(a_unsigned_range).to_downto(10, 0)?
            )
            .declare("", ";\n")?
        );
        print!(
            "{}",
            AssignDeclaration::new(test_complex_signal()?, a_signed.clone().into())
                .declare("", ";\n")?
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
                .assign(
                    &Assignment::from(a_signed.clone())
                        .to_named("a")
                        .to_downto(4, -3)?
                )?
                .declare("", ";\n")?
        );
        print!(
            "{}",
            AssignDeclaration::new(
                test_complex_signal()?,
                Assignment::from(a_signed_range).to_to(0, 10)?
            )
            .declare("", ";\n")?
        );
        print!(
            "{}",
            AssignDeclaration::new(test_complex_signal()?, a_str.into()).declare("", ";\n")?
        );
        Ok(())
    }

    #[test]
    fn test_record_assign() -> Result<()> {
        let a_single = BitVecValue::Others(StdLogicValue::H);
        let mut multifields = IndexMap::new();
        multifields.insert(
            "c".to_string(),
            BitVecValue::Others(StdLogicValue::H).into(),
        );
        multifields.insert("d".to_string(), BitVecValue::Signed(-55).into());
        let a_full = AssignmentKind::full_record(multifields);
        assert_eq!(
            "recname.c <= (others => 'H');\n",
            test_record_signal("rectype", "recname")?
                .assign(&Assignment::from(a_single.clone()).to_named("c"))?
                .declare("", ";\n")?
        );
        assert_eq!(
            "recname2.c(40 downto 30) <= (others => 'H');\n",
            test_record_signal("rectype", "recname2")?
                .assign(
                    &Assignment::from(a_single.clone())
                        .to_named("c")
                        .to_downto(40, 30)?
                )?
                .declare("", ";\n")?
        );
        assert_eq!(
            r#"  recname3 <= (
    c => (others => 'H'),
    d => std_logic_vector(to_signed(-55, recname3.d'length))
  );
"#,
            test_record_signal("rectype", "recname3")?
                .assign(&a_full)?
                .declare("  ", ";\n")?
        );
        Ok(())
    }

    #[test]
    fn print_array_assign() -> Result<()> {
        print!(
            "{}",
            test_bitvec_signal("recname", 10, 0)?
                .assign(&Assignment::from(StdLogicValue::U).to_index(4))?
                .declare("", ";\n")?
        );
        print!(
            "{}",
            test_bitvec_signal("recname", 8, 0)?
                .assign(&BitVecValue::from_str("10ZWUHLX-")?)?
                .declare("", ";\n")?
        );

        Ok(())
    }
}
