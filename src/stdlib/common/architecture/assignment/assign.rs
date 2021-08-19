use super::{Assign, AssignedObject, Assignment};
use crate::{Result, stdlib::common::architecture::declaration::ObjectDeclaration};

impl Assign for ObjectDeclaration {
    fn assign(&self, assignment: &Assignment) -> Result<AssignedObject> {
        self.typ().can_assign(assignment)?;
        Ok(AssignedObject::new(self.clone(), assignment.clone()))
    }
}