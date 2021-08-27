use super::{AssignDeclaration, Assignment, FieldSelection};
use crate::{
    stdlib::common::architecture::{
        assignment::{Assign, ObjectAssignment},
        declaration::ObjectDeclaration,
        object::ObjectType,
    },
    Error, Result,
};

pub trait FlatLength {
    /// Returns the total length of the object when flattened
    fn flat_length(&self) -> u32;
    /// Returns the total length of a field of the object
    fn flat_length_for(&self, nested_fields: &Vec<FieldSelection>) -> Result<u32>;
}

/// This trait enables connecting complex objects and flat (bit/bit vector) objects
/// by selecting to individual bit/bit vector fields across multiple assignments.
pub trait FlatAssignment: FlatLength {
    /// Assigns a flat object to a complex object over multiple assignments
    fn from_flat(
        &self,
        flat_object: &ObjectDeclaration,
        to_field: &Vec<FieldSelection>,
        from_field: &FieldSelection,
    ) -> Result<Vec<AssignDeclaration>>;

    /// Assigns a complex object to a flat object over multiple assignments
    fn from_complex(
        &self,
        complex_object: &ObjectDeclaration,
        to_field: &FieldSelection,
        from_field: &Vec<FieldSelection>,
    ) -> Result<Vec<AssignDeclaration>>;
}

impl FlatLength for ObjectType {
    fn flat_length(&self) -> u32 {
        match self {
            ObjectType::Bit => 1,
            ObjectType::Array(arr) => arr.width() * arr.typ().flat_length(),
            ObjectType::Record(rec) => {
                let mut total: u32 = 0;
                for (_, typ) in rec.fields() {
                    total += typ.flat_length();
                }
                total
            }
        }
    }

    fn flat_length_for(&self, nested_fields: &Vec<FieldSelection>) -> Result<u32> {
        Ok(self.get_nested(nested_fields)?.flat_length())
    }
}

impl FlatLength for ObjectDeclaration {
    fn flat_length(&self) -> u32 {
        self.typ().flat_length()
    }

    fn flat_length_for(&self, nested_fields: &Vec<FieldSelection>) -> Result<u32> {
        self.typ().flat_length_for(nested_fields)
    }
}

impl FlatAssignment for ObjectDeclaration {
    fn from_flat(
        &self,
        _flat_object: &ObjectDeclaration,
        _to_field: &Vec<FieldSelection>,
        _from_field: &FieldSelection,
    ) -> Result<Vec<AssignDeclaration>> {
        todo!()
    }

    fn from_complex(
        &self,
        complex_object: &ObjectDeclaration,
        to_field: &FieldSelection,
        from_field: &Vec<FieldSelection>,
    ) -> Result<Vec<AssignDeclaration>> {
        // TODO: When length is 1, make sure the field selection matches. (So we don't assign a range (0 downto 0) to a bit, for example)
        if self.flat_length_for(&vec![to_field.clone()])?
            != complex_object.flat_length_for(from_field)?
        {
            Err(Error::InvalidArgument(format!("Can't assign objects to one another, mismatched length (self: {}, complex object: {})", self.flat_length(), complex_object.flat_length())))
        } else {
            let mut result = vec![];
            let finish = result.push(
                self.assign(
                    &Assignment::from(
                        ObjectAssignment::from(complex_object.clone()).assign_from(from_field)?,
                    )
                    .to(to_field.clone()),
                )?,
            );
            match complex_object.typ().get_field(to_field)? {
                ObjectType::Bit => {
                    finish;
                }
                ObjectType::Array(arr) if arr.is_bitvector() => {
                    finish;
                }
                ObjectType::Array(_arr) => {
                    // TODO: WIP: for each element of the array, check its length, then do a to_flat for the range equivalent to that length to a subsection of the flat object
                    // If the length is 1, make it an index field selection,
                }
                ObjectType::Record(_) => todo!(),
            }
            Ok(result)
        }
    }
}
