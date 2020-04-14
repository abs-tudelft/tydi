use indexmap::map::IndexMap;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
enum Type<'t> {
    Bits(usize),
    Group(IndexMap<String, Type<'t>>),
    Ref(String, String, RefCell<Option<&'t Type<'t>>>),
}

impl<'p> Type<'p> {
    fn new_ref(lib: impl Into<String>, typ: impl Into<String>) -> Self {
        Type::Ref(lib.into(), typ.into(), RefCell::new(None))
    }

    fn is_resolved(&self) -> bool {
        match self {
            Type::Bits(_) => true,
            Type::Group(inner) => inner.iter().all(|(_, t)| t.is_resolved()),
            Type::Ref(_, _, reference) => reference.borrow().is_some(),
        }
    }

    fn resolve(&self, prj: &'p Project<'p>, trace: &mut Vec<(String, String)>) {
        match self {
            Type::Bits(_) => {}
            Type::Group(inner) => {
                for (_, field) in inner {
                    field.resolve(prj, trace)
                }
            }
            Type::Ref(lib, key, reference) => {
                println!("Resolving ref to: {}::{}", lib, key);
                if reference.borrow().is_none() {
                    trace.iter().for_each(|k| {
                        if &k.0 == lib && &k.1 == key {
                            panic!("Type hierarchy contains circular reference...")
                        }
                    });
                    // This type reference has not yet been resolved.
                    let t = prj.get_type((lib, key));
                    if let Some(r) = t {
                        // Resolve the referenced type recursively.
                        trace.push((lib.clone(), key.clone()));

                        r.resolve(prj, trace);
                        // Internally mutate the reference.
                        *reference.borrow_mut() = Some(r);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Library<'l> {
    types: IndexMap<String, Type<'l>>,
}

impl<'l> Library<'l> {
    fn types(&self) -> impl Iterator<Item = (&String, &Type<'l>)> {
        self.types.iter()
    }

    fn get_type(&self, key: &String) -> Option<&Type<'l>> {
        self.types.get(key)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Project<'p> {
    libs: IndexMap<String, Library<'p>>,
}

impl<'p> Project<'p> {
    fn types(&'p self) -> impl IntoIterator<Item = (&String, &String, &'p Type<'p>)> {
        let mut result = Vec::new();
        for (lib_key, lib) in self.libs.iter() {
            for (type_key, typ) in lib.types() {
                result.push((lib_key, type_key, typ));
            }
        }
        result.into_iter()
    }

    fn get_type(&'p self, key: (&String, &String)) -> Option<&'p Type<'p>> {
        self.libs.get(key.0).and_then(|l| l.get_type(key.1))
    }

    fn resolve_types(&'p self) {
        for (l, k, t) in self.types() {
            if !t.is_resolved() {
                dbg!(&t);
                // Trace to detect circular references:
                let mut trace = vec![(l.clone(), k.clone())];
                t.resolve(self, &mut trace)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok() {
        let prj = Project {
            libs: vec![
                (
                    "k".to_string(),
                    Library {
                        types: vec![
                            ("A".to_string(), Type::Bits(1)),
                            ("B".to_string(), Type::new_ref("k", "A")),
                            (
                                "C".to_string(),
                                Type::Group(
                                    vec![
                                        ("x".to_string(), Type::new_ref("k", "A")),
                                        ("y".to_string(), Type::new_ref("k", "B")),
                                    ]
                                    .into_iter()
                                    .collect(),
                                ),
                            ),
                            ("E".to_string(), Type::new_ref("k", "D")),
                            ("D".to_string(), Type::new_ref("k", "C")),
                        ]
                        .into_iter()
                        .collect(),
                    },
                ),
                (
                    "l".to_string(),
                    Library {
                        types: vec![("E".to_string(), Type::new_ref("k", "D"))]
                            .into_iter()
                            .collect(),
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        prj.resolve_types();

        dbg!(&prj);
    }

    #[test]
    fn test_circular_reference() {
        let prj = Project {
            libs: vec![
                (
                    "k".to_string(),
                    Library {
                        types: vec![
                            ("A".to_string(), Type::Bits(1)),
                            ("B".to_string(), Type::new_ref("k", "A")),
                            (
                                "C".to_string(),
                                Type::Group(
                                    vec![
                                        ("x".to_string(), Type::new_ref("k", "A")),
                                        ("y".to_string(), Type::new_ref("k", "B")),
                                    ]
                                    .into_iter()
                                    .collect(),
                                ),
                            ),
                            ("D".to_string(), Type::new_ref("l", "E")),
                        ]
                        .into_iter()
                        .collect(),
                    },
                ),
                (
                    "l".to_string(),
                    Library {
                        types: vec![("E".to_string(), Type::new_ref("k", "D"))]
                            .into_iter()
                            .collect(),
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        prj.resolve_types();

        dbg!(&prj);
    }
}
