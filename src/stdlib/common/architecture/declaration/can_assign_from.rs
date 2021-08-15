use crate::stdlib::common::architecture::assignment::{Assignment, CanAssignFrom};
use crate::Result;

use super::{ObjectDeclaration, ObjectKind};

impl CanAssignFrom for ObjectDeclaration {
    fn can_assign_from(&self, assignment: &Assignment) -> Result<()> {
        match assignment {
            Assignment::Object(object) => todo!(),
            Assignment::Value(value) => todo!(),
        }
    }
}
