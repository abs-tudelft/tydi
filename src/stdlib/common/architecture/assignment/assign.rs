use super::{Assign, AssignedObject, Assignment};
use crate::{Result, stdlib::common::architecture::declaration::ObjectDeclaration};

impl Assign for ObjectDeclaration {
    fn assign(&self, assignment: Assignment) -> Result<AssignedObject> {
        todo!()
    }
}