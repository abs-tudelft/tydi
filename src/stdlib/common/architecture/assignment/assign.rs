use super::{Assign, AssignedObject, AssignmentKind};
use crate::{Result, stdlib::common::architecture::declaration::ObjectDeclaration};

impl Assign for ObjectDeclaration {
    fn assign(&self, assignment: AssignmentKind) -> Result<AssignedObject> {
        todo!()
    }
}