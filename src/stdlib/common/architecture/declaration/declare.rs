use crate::Result;
use crate::stdlib::common::architecture::ArchitectureDeclare;

use super::{ArchitectureDeclaration, ObjectDeclaration, ObjectKind};

impl ArchitectureDeclare for ArchitectureDeclaration<'_> {
    fn declare(&self, pre: &str, post: &str) -> crate::Result<String> {
        match self {
            ArchitectureDeclaration::Type(_) => todo!(),
            ArchitectureDeclaration::SubType(_) => todo!(),
            ArchitectureDeclaration::Procedure(_) => todo!(),
            ArchitectureDeclaration::Function(_) => todo!(),
            ArchitectureDeclaration::Object(object) => object.declare(pre, post),
            ArchitectureDeclaration::Alias(_) => todo!(),
            ArchitectureDeclaration::Component(_) => todo!(),
            ArchitectureDeclaration::Custom(_) => todo!(),
        }
    }
}

impl ArchitectureDeclare for ObjectDeclaration {
    fn declare(&self, pre: &str, post: &str) -> Result<String> {
        if self.kind() == ObjectKind::EntityPort {
            // Entity ports are part of the architecture, but aren't declared in the declaration part
            return Ok("".to_string());
        }
        let mut result = pre.to_string();
        result.push_str(match self.kind() {
            ObjectKind::Signal => "signal ",
            ObjectKind::Variable => "variable ",
            ObjectKind::Constant => "constant ",
            ObjectKind::EntityPort => "", // Should be unreachable
            ObjectKind::ComponentPort => "",
        });
        result.push_str(&self.identifier());
        result.push_str(" : ");
        result.push_str(self.typ().type_name());
        result.push_str(match self.mode() {
            super::ObjectMode::Undefined => "",
            super::ObjectMode::Assigned => "out ",
            super::ObjectMode::Out => "in ",
        });
        if let Some(default) = self.default() {
            result.push_str(" := ");
            result.push_str(
                &default
                    .declare_for(self.identifier())?
                    .replace("##pre##", pre),
            );
        }
        result.push_str(post);
        result.push_str("\n");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::stdlib::common::architecture::{assignment::StdLogicValue, object::ObjectType};

    use super::*;

    #[test]
    fn test_declarations() -> Result<()> {
        assert_eq!(
            "signal TestSignal : std_logic;\n",
            ObjectDeclaration::signal("TestSignal", ObjectType::Bit, None).declare("", ";")?
        );
        assert_eq!(
            "variable TestVariable : std_logic;\n",
            ObjectDeclaration::variable("TestVariable", ObjectType::Bit, None).declare("", ";")?
        );
        assert_eq!(
            "signal SignalWithDefault : std_logic := 'U';\n",
            ObjectDeclaration::signal(
                "SignalWithDefault",
                ObjectType::Bit,
                Some(StdLogicValue::U.into())
            )
            .declare("", ";")?
        );
        assert_eq!(
            "  constant TestConstant : std_logic := 'U';\n",
            ObjectDeclaration::constant("TestConstant", ObjectType::Bit, StdLogicValue::U)
                .declare("  ", ";")?
        );
        Ok(())
    }
}
