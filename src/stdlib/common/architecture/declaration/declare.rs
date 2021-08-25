use crate::Result;
use crate::{stdlib::common::architecture::ArchitectureDeclare, Error};

use super::{ArchitectureDeclaration, ObjectDeclaration, ObjectKind, ObjectMode};

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
        if self.kind() == ObjectKind::ComponentPort {
            match self.mode() {
                ObjectMode::Undefined => {
                    return Err(Error::BackEndError(format!(
                        "Component port {} has no direction",
                        self.identifier()
                    )));
                }
                ObjectMode::Assigned => result.push_str("out "),
                ObjectMode::Out => result.push_str("in "),
            };
        }
        result.push_str(self.typ().type_name());
        if let Some(default) = self.default() {
            result.push_str(" := ");
            result.push_str(&default.declare_for(self.identifier(), pre, post)?);
        }
        result.push_str(post);
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
            ObjectDeclaration::signal("TestSignal", ObjectType::Bit, None).declare("", ";\n")?
        );
        assert_eq!(
            "variable TestVariable : std_logic;\n",
            ObjectDeclaration::variable("TestVariable", ObjectType::Bit, None)
                .declare("", ";\n")?
        );
        assert_eq!(
            "signal SignalWithDefault : std_logic := 'U';\n",
            ObjectDeclaration::signal(
                "SignalWithDefault",
                ObjectType::Bit,
                Some(StdLogicValue::U.into())
            )
            .declare("", ";\n")?
        );
        assert_eq!(
            "  constant TestConstant : std_logic := 'U';\n",
            ObjectDeclaration::constant("TestConstant", ObjectType::Bit, StdLogicValue::U)
                .declare("  ", ";\n")?
        );
        Ok(())
    }
}
