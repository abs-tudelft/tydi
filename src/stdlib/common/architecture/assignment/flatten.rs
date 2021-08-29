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
    fn flat_length(&self) -> Result<u32>;
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
    fn flat_length(&self) -> Result<u32> {
        Ok(match self {
            ObjectType::Bit => 1,
            ObjectType::Array(arr) => arr.width() * arr.typ().flat_length()?,
            ObjectType::Record(rec) => {
                let mut total: u32 = 0;
                if rec.is_union() {
                    let mut max: u32 = 0;
                    for (name, typ) in rec.fields() {
                        if name != "tag" && typ.flat_length()? > max {
                            max = typ.flat_length()?;
                        }
                    }
                    total += max;
                    total += rec.get_field("tag")?.flat_length()?;
                } else {
                    for (_, typ) in rec.fields() {
                        total += typ.flat_length()?;
                    }
                }
                total
            }
        })
    }

    fn flat_length_for(&self, nested_fields: &Vec<FieldSelection>) -> Result<u32> {
        self.get_nested(nested_fields)?.flat_length()
    }
}

impl FlatLength for ObjectDeclaration {
    fn flat_length(&self) -> Result<u32> {
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
        let self_typ = self.typ().get_nested(from_field)?;
        if !self_typ.is_flat() {
            Err(Error::InvalidArgument(format!(
                "self ({}{}) must be flat, is a {} instead",
                self.identifier(),
                print_fields(from_field),
                self_typ
            )))
        } else {
            let complex_typ = complex_object.typ().get_nested(to_field)?;
            match &complex_typ {
                ObjectType::Record(rec) if rec.is_union() => {
                    let self_typ = self.typ().get_nested(from_field)?;
                    let tag_length = rec.get_field("tag")?.flat_length()?;
                    let remainder = complex_typ.flat_length()? - tag_length;
                    let mut result = vec![];
                    for (name, field) in rec.fields() {
                        let mut new_to = to_field.clone();
                        let mut new_from = from_field.clone();
                        new_to.push(FieldSelection::name(name));
                        if name == "tag" {
                            select_specific_flat_range(
                                &mut new_from,
                                remainder,
                                tag_length,
                                &self_typ,
                            )?;
                        } else {
                            select_specific_flat_range(
                                &mut new_from,
                                0,
                                field.flat_length()?,
                                &self_typ,
                            )?;
                        }
                        result.extend(self.to_complex(complex_object, &new_to, &new_from)?);
                    }
                    Ok(result)
                }
                _ => complex_object
                    .to_flat(self, from_field, to_field)?
                    .iter()
                    .map(|x| x.reverse())
                    .collect(),
            }
        }
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
            Err(Error::InvalidArgument(format!("Can't assign objects to one another, mismatched length (self ({}{}): {}, flat object ({}{}): {})",
             self.identifier(), print_fields(from_field), self.flat_length_for(from_field)?,
             flat_object.identifier(), print_fields(to_field), flat_object.flat_length_for(to_field)?)))
        } else if !flat_typ.is_flat() {
            Err(Error::InvalidArgument(format!(
                "flat_object ({}{}) must be flat, is a {} instead",
                flat_object.identifier(),
                print_fields(to_field),
                flat_typ
            )))
        } else {
            let mut result = vec![];
            let mut finalize = || -> Result<()> {
                let mut new_from = from_field.clone();
                let mut new_to = to_field.clone();
                // If the length == 1 and one object is a Bit, make sure that both select a Bit (avoid left(1) <= right(0 downto 0))
                if self_typ.flat_length()? == 1 {
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
                            let typ_length = arr.typ().flat_length()?;
                            let mut new_from = from_field.clone();
                            new_from.push(FieldSelection::index(index));
                            // Subdivide the range selection on the flat object to match the length of each field of the complex object
                            let mut new_to = to_field.clone();
                            select_flat_range(
                                &mut new_to,
                                normalized_index,
                                typ_length,
                                &flat_typ,
                            )?;
                            result.extend(self.to_flat(flat_object, &new_to, &new_from)?);
                        }
                    }
                }
                ObjectType::Record(rec) => {
                    if rec.is_union() {
                        // TODO: This is incorrect. Driving the signal from multiple sources doesn't work.
                        // Ideally, figure out some way to generate an "or" on multiple signals
                        let tag_length = rec.get_field("tag")?.flat_length()?;
                        let remainder = self_typ.flat_length()? - tag_length;
                        for (name, field) in rec.fields() {
                            let mut new_to = to_field.clone();
                            let mut new_from = from_field.clone();
                            new_from.push(FieldSelection::name(name));
                            if name == "tag" {
                                select_specific_flat_range(
                                    &mut new_to,
                                    remainder,
                                    tag_length,
                                    &flat_typ,
                                )?;
                            } else {
                                select_specific_flat_range(
                                    &mut new_to,
                                    0,
                                    field.flat_length()?,
                                    &flat_typ,
                                )?;
                            }
                            result.extend(self.to_flat(flat_object, &new_to, &new_from)?);
                        }
                    } else {
                        let mut preceding_length = 0;
                        for (name, field) in rec.fields() {
                            let mut new_to = to_field.clone();
                            let mut new_from = from_field.clone();
                            new_from.push(FieldSelection::name(name));
                            let field_length = field.flat_length()?;
                            select_specific_flat_range(
                                &mut new_to,
                                preceding_length,
                                field_length,
                                &flat_typ,
                            )?;
                            result.extend(self.to_flat(flat_object, &new_to, &new_from)?);
                            preceding_length += field_length;
                        }
                    }
                }
            }
            Ok(result)
        }
    }
}

fn print_fields(fields: &Vec<FieldSelection>) -> String {
    if fields.len() > 0 {
        let field_strings: Vec<String> = fields.iter().map(|x| x.to_string()).collect();
        field_strings.join("")
    } else {
        "".to_string()
    }
}

/// Selects a range on a flat object. If a range is already selected, instead replaces that selection with the smaller range.
///
/// Assumes current selection is an array
fn select_flat_range(
    current_selection: &mut Vec<FieldSelection>,
    normalized_index: u32,
    typ_length: u32,
    flat_typ: &ObjectType,
) -> Result<()> {
    select_specific_flat_range(
        current_selection,
        normalized_index * typ_length,
        typ_length,
        flat_typ,
    )
}

/// Selects a range on a flat object. If a range is already selected, instead replaces that selection with the smaller range.
///
/// Requires the length of all preceding elements and the length of the current object
fn select_specific_flat_range(
    current_selection: &mut Vec<FieldSelection>,
    preceding_length: u32,
    curr_length: u32,
    flat_typ: &ObjectType,
) -> Result<()> {
    if let Some(some) = current_selection.last_mut() {
        if let FieldSelection::Range(range) = some {
            *range = sub_range(range.low(), preceding_length, curr_length)?;
        } else {
            unreachable!()
        }
    } else {
        if let ObjectType::Array(flat_arr) = flat_typ {
            current_selection.push(FieldSelection::Range(sub_range(
                flat_arr.low(),
                preceding_length,
                curr_length,
            )?));
        } else {
            unreachable!()
        }
    };
    Ok(())
}

fn sub_range(min_range: i32, preceding_length: u32, curr_length: u32) -> Result<RangeConstraint> {
    RangeConstraint::downto(
        min_range + i32::try_from(preceding_length + curr_length).unwrap() - 1,
        min_range + i32::try_from(preceding_length).unwrap(),
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
    use crate::generator::common::test::records;
    use crate::stdlib::common::architecture::assignment::declare::tests::nested_record_signal;
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
            r#"flat(9 downto 0) <= complex(-2)(-1);
flat(19 downto 10) <= complex(-2)(0);
flat(29 downto 20) <= complex(-1)(-1);
flat(39 downto 30) <= complex(-1)(0);
flat(49 downto 40) <= complex(0)(-1);
flat(59 downto 50) <= complex(0)(0);
flat(69 downto 60) <= complex(1)(-1);
flat(79 downto 70) <= complex(1)(0);
flat(89 downto 80) <= complex(2)(-1);
flat(99 downto 90) <= complex(2)(0);
"#
        );
        let to_complex_assignments = flat.to_complex(&complex, &vec![], &vec![])?;
        let mut full_complex = String::new();
        for a in to_complex_assignments {
            full_complex.push_str(&a.declare("", ";\n")?)
        }
        assert_eq!(
            full_complex,
            r#"complex(-2)(-1) <= flat(9 downto 0);
complex(-2)(0) <= flat(19 downto 10);
complex(-1)(-1) <= flat(29 downto 20);
complex(-1)(0) <= flat(39 downto 30);
complex(0)(-1) <= flat(49 downto 40);
complex(0)(0) <= flat(59 downto 50);
complex(1)(-1) <= flat(69 downto 60);
complex(1)(0) <= flat(79 downto 70);
complex(2)(-1) <= flat(89 downto 80);
complex(2)(0) <= flat(99 downto 90);
"#
        );
        Ok(())
    }

    #[test]
    fn test_record_flatten() -> Result<()> {
        let record = nested_record_signal("rec_type", "rec")?;
        let flat = ObjectDeclaration::signal("flat", ObjectType::bit_vector(2757, 0)?, None);
        //print!("{} {}\n", record.flat_length()?, flat.flat_length()?);
        let to_flat_assignments = record.to_flat(&flat, &vec![], &vec![])?;
        let mut full_flat = String::new();
        for a in to_flat_assignments {
            full_flat.push_str(&a.declare("", ";\n")?)
        }
        assert_eq!(
            r#"flat(41 downto 0) <= rec.a.c;
flat(1378 downto 42) <= rec.a.d;
flat(1420 downto 1379) <= rec.b.c;
flat(2757 downto 1421) <= rec.b.d;
"#,
            full_flat
        );
        let to_complex_assignments = flat.to_complex(&record, &vec![], &vec![])?;
        let mut full_complex = String::new();
        for a in to_complex_assignments {
            full_complex.push_str(&a.declare("", ";\n")?)
        }
        assert_eq!(
            r#"rec.a.c <= flat(41 downto 0);
rec.a.d <= flat(1378 downto 42);
rec.b.c <= flat(1420 downto 1379);
rec.b.d <= flat(2757 downto 1421);
"#,
            full_complex
        );
        Ok(())
    }

    #[test]
    fn test_union_flatten() -> Result<()> {
        let union = ObjectDeclaration::signal("union", records::union("union_t").try_into()?, None);
        let flat = ObjectDeclaration::signal("flat", ObjectType::bit_vector(1338, 0)?, None);
        //print!("{} {}\n", union.flat_length()?, flat.flat_length()?);
        let to_flat_assignments = union.to_flat(&flat, &vec![], &vec![])?;
        let mut full_flat = String::new();
        for a in to_flat_assignments {
            full_flat.push_str(&a.declare("", ";\n")?)
        }
        assert_eq!(
            r#"flat(1338 downto 1337) <= union.tag;
flat(41 downto 0) <= union.c;
flat(1336 downto 0) <= union.d;
"#,
            full_flat
        );
        let to_complex_assignments = flat.to_complex(&union, &vec![], &vec![])?;
        let mut full_complex = String::new();
        for a in to_complex_assignments {
            full_complex.push_str(&a.declare("", ";\n")?)
        }
        assert_eq!(
            r#"union.tag <= flat(1338 downto 1337);
union.c <= flat(41 downto 0);
union.d <= flat(1336 downto 0);
"#,
            full_complex
        );
        Ok(())
    }

    #[test]
    fn test_nested_union_flatten() -> Result<()> {
        let union =
            ObjectDeclaration::signal("union", records::union_nested("union_t").try_into()?, None);
        let flat = ObjectDeclaration::signal("flat", ObjectType::bit_vector(1340, 0)?, None);
        print!("{} {}\n", union.flat_length()?, flat.flat_length()?);
        let to_flat_assignments = union.to_flat(&flat, &vec![], &vec![])?;
        let mut full_flat = String::new();
        for a in to_flat_assignments {
            full_flat.push_str(&a.declare("", ";\n")?)
        }
        print!("{}", full_flat);
        let to_complex_assignments = flat.to_complex(&union, &vec![], &vec![])?;
        let mut full_complex = String::new();
        for a in to_complex_assignments {
            full_complex.push_str(&a.declare("", ";\n")?)
        }
        print!("{}", full_complex);
        Ok(())
    }
}
