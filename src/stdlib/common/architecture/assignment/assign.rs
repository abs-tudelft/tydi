use super::{AssignedObject, Assignment};
use crate::{Result, stdlib::common::architecture::declaration::ObjectDeclaration};

pub trait Assign {
    fn assign(&self, assignment: Assignment) -> Result<AssignedObject>;
}

impl Assign for ObjectDeclaration {
    fn assign(&self, assignment: Assignment) -> Result<AssignedObject> {
        todo!()
    }
}