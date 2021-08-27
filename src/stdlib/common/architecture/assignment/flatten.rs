use std::convert::{TryFrom, TryInto};

use super::{AssignDeclaration, Assignment, FieldSelection};
use crate::{
    stdlib::common::architecture::{
        assignment::{Assign, ObjectAssignment, RangeConstraint},
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
    fn to_complex(
        &self,
        complex_object: &ObjectDeclaration,
        to_field: &Vec<FieldSelection>,
        from_field: &Vec<FieldSelection>,
    ) -> Result<Vec<AssignDeclaration>>;

    /// Assigns a complex object to a flat object over multiple assignments
    fn to_flat(
        &self,
        flat_object: &ObjectDeclaration,
        to_field: &Vec<FieldSelection>,
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
    fn to_complex(
        &self,
        complex_object: &ObjectDeclaration,
        to_field: &Vec<FieldSelection>,
        from_field: &Vec<FieldSelection>,
    ) -> Result<Vec<AssignDeclaration>> {
        complex_object
            .to_flat(self, from_field, to_field)?
            .iter()
            .map(|x| x.reverse())
            .collect()
    }

    fn to_flat(
        &self,
        flat_object: &ObjectDeclaration,
        to_field: &Vec<FieldSelection>,
        from_field: &Vec<FieldSelection>,
    ) -> Result<Vec<AssignDeclaration>> {
        let self_typ = self.typ().get_nested(from_field)?;
        let flat_typ = flat_object.typ().get_nested(to_field)?;
        if self_typ.flat_length() != flat_typ.flat_length() {
            Err(Error::InvalidArgument(format!("Can't assign objects to one another, mismatched length (self: {}, flat object: {})", self.flat_length_for(from_field)?, flat_object.flat_length_for(to_field)?)))
        } else if !flat_typ.is_flat() {
            Err(Error::InvalidArgument(format!(
                "flat_object must be flat, is a {} instead",
                flat_typ
            )))
        } else {
            let mut result = vec![];
            let mut finalize = || -> Result<()> {
                let mut new_from = from_field.clone();
                let mut new_to = to_field.clone();
                // If the length == 1 and one object is a Bit, make sure that both select a Bit (avoid left(1) <= right(0 downto 0))
                if self_typ.flat_length() == 1 {
                    match_bit_field_selection(&flat_typ, &self_typ, &mut new_from);
                    match_bit_field_selection(&self_typ, &flat_typ, &mut new_to);
                }
                result.push(
                    flat_object.assign(
                        &Assignment::from(
                            ObjectAssignment::from(self.clone()).assign_from(&new_from)?,
                        )
                        .to_nested(&new_to),
                    )?,
                );
                Ok(())
            };
            match &self_typ {
                ObjectType::Bit => finalize()?,
                ObjectType::Array(arr) if arr.is_bitvector() => finalize()?,
                ObjectType::Array(arr) => {
                    // If the length is 1, make the last range selection an index selection, or introduce an index selection
                    if arr.width() == 1 {
                        let mut new_from = from_field.clone();
                        if let Some(some) = new_from.last_mut() {
                            if let FieldSelection::Range(range) = some {
                                *range = RangeConstraint::Index(range.high());
                            } else {
                                unreachable!()
                            }
                        } else {
                            new_from.push(FieldSelection::index(arr.high()));
                        };
                        result.extend(self.to_flat(flat_object, to_field, &new_from)?);
                    } else {
                        for index in arr.low()..(arr.high() + 1) {
                            let normalized_index = (index - arr.low()) as u32;
                            let typ_length = arr.typ().flat_length();
                            let mut new_from = from_field.clone();
                            new_from.push(FieldSelection::index(index));
                            // Subdivide the range selection on the flat object to match the length of each field of the complex object
                            let mut new_to = to_field.clone();
                            if let Some(some) = new_to.last_mut() {
                                if let FieldSelection::Range(range) = some {
                                    *range = sub_range(range.high(), normalized_index, typ_length)?;
                                } else {
                                    unreachable!()
                                }
                            } else {
                                if let ObjectType::Array(flat_arr) = &flat_typ {
                                    new_to.push(FieldSelection::Range(sub_range(
                                        flat_arr.high(),
                                        normalized_index,
                                        typ_length,
                                    )?));
                                } else {
                                    unreachable!()
                                }
                            };
                            result.extend(self.to_flat(flat_object, &new_to, &new_from)?);
                        }
                    }
                }
                ObjectType::Record(_) => todo!(),
            }
            Ok(result)
        }
    }
}

fn sub_range(max_range: i32, normalized_index: u32, typ_length: u32) -> Result<RangeConstraint> {
    RangeConstraint::downto(
        max_range - i32::try_from(normalized_index * typ_length).unwrap(),
        max_range - i32::try_from((normalized_index + 1) * typ_length).unwrap() + 1,
    )
}

/// If `to` is a Bit and `from` is a Bit Vector, use an index selection for `from`
fn match_bit_field_selection(
    left: &ObjectType,
    right: &ObjectType,
    right_selection: &mut Vec<FieldSelection>,
) {
    if let ObjectType::Bit = left {
        if let ObjectType::Array(arr) = right {
            if let Some(some) = right_selection.last_mut() {
                if let FieldSelection::Range(range) = some {
                    *range = RangeConstraint::Index(range.high());
                } else {
                    unreachable!()
                }
            } else {
                right_selection.push(FieldSelection::index(arr.high()));
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::stdlib::common::architecture::ArchitectureDeclare;

    use super::*;

    #[test]
    fn test_array_flatten() -> Result<()> {
        let complex_array = ObjectType::array(
            2,
            -2,
            ObjectType::array(
                0,
                -1,
                ObjectType::bit_vector(9, 0)?,
                "some_inner_array_type",
            )?,
            "some_array_type",
        )?;
        let flat_vector = ObjectType::bit_vector(99, 0)?;
        let complex = ObjectDeclaration::signal("complex", complex_array, None);
        let flat = ObjectDeclaration::signal("flat", flat_vector, None);
        let to_flat_assignments = complex.to_flat(&flat, &vec![], &vec![])?;
        let mut full_flat = String::new();
        for a in to_flat_assignments {
            full_flat.push_str(&a.declare("", ";\n")?)
        }
        assert_eq!(
            full_flat,
            r#"flat(99 downto 90) <= complex(-2)(-1);
flat(89 downto 80) <= complex(-2)(0);
flat(79 downto 70) <= complex(-1)(-1);
flat(69 downto 60) <= complex(-1)(0);
flat(59 downto 50) <= complex(0)(-1);
flat(49 downto 40) <= complex(0)(0);
flat(39 downto 30) <= complex(1)(-1);
flat(29 downto 20) <= complex(1)(0);
flat(19 downto 10) <= complex(2)(-1);
flat(9 downto 0) <= complex(2)(0);
"#
        );
        let to_complex_assignments = flat.to_complex(&complex, &vec![], &vec![])?;
        let mut full_complex = String::new();
        for a in to_complex_assignments {
            full_complex.push_str(&a.declare("", ";\n")?)
        }
        assert_eq!(
            full_complex,
            r#"complex(-2)(-1) <= flat(99 downto 90);
complex(-2)(0) <= flat(89 downto 80);
complex(-1)(-1) <= flat(79 downto 70);
complex(-1)(0) <= flat(69 downto 60);
complex(0)(-1) <= flat(59 downto 50);
complex(0)(0) <= flat(49 downto 40);
complex(1)(-1) <= flat(39 downto 30);
complex(1)(0) <= flat(29 downto 20);
complex(2)(-1) <= flat(19 downto 10);
complex(2)(0) <= flat(9 downto 0);
"#
        );
        Ok(())
    }
}
