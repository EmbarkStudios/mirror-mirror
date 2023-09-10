use crate::{
    enum_::{VariantField, VariantKind},
    Array, Enum, List, Map, Reflect, ReflectRef, Struct, Tuple, TupleStruct,
};

/// Compare two reflected values for equality.
///
/// Returns `None` if either value contains a `ReflectRef::Opaque`.
pub fn reflect_eq(a: &dyn Reflect, b: &dyn Reflect) -> Option<bool> {
    match (a.reflect_ref(), b.reflect_ref()) {
        (ReflectRef::Scalar(a), ReflectRef::Scalar(b)) => Some(a == b),
        (ReflectRef::Struct(a), ReflectRef::Struct(b)) => reflect_eq_struct(a, b),
        (ReflectRef::TupleStruct(a), ReflectRef::TupleStruct(b)) => reflect_eq_tuple_struct(a, b),
        (ReflectRef::Tuple(a), ReflectRef::Tuple(b)) => reflect_eq_tuple(a, b),
        (ReflectRef::Enum(a), ReflectRef::Enum(b)) => reflect_eq_enum(a, b),
        (ReflectRef::Array(a), ReflectRef::Array(b)) => reflect_eq_array(a, b),
        (ReflectRef::List(a), ReflectRef::List(b)) => reflect_eq_list(a, b),
        (ReflectRef::Map(a), ReflectRef::Map(b)) => reflect_eq_map(a, b),
        (ReflectRef::Opaque(_), _) | (_, ReflectRef::Opaque(_)) => None,

        (
            ReflectRef::Struct(_)
            | ReflectRef::Tuple(_)
            | ReflectRef::Enum(_)
            | ReflectRef::Array(_)
            | ReflectRef::List(_)
            | ReflectRef::Map(_)
            | ReflectRef::Scalar(_),
            ReflectRef::TupleStruct(_),
        )
        | (
            ReflectRef::Struct(_)
            | ReflectRef::TupleStruct(_)
            | ReflectRef::Enum(_)
            | ReflectRef::Array(_)
            | ReflectRef::List(_)
            | ReflectRef::Map(_)
            | ReflectRef::Scalar(_),
            ReflectRef::Tuple(_),
        )
        | (
            ReflectRef::Struct(_)
            | ReflectRef::TupleStruct(_)
            | ReflectRef::Tuple(_)
            | ReflectRef::Array(_)
            | ReflectRef::List(_)
            | ReflectRef::Map(_)
            | ReflectRef::Scalar(_),
            ReflectRef::Enum(_),
        )
        | (
            ReflectRef::Struct(_)
            | ReflectRef::TupleStruct(_)
            | ReflectRef::Tuple(_)
            | ReflectRef::Enum(_)
            | ReflectRef::List(_)
            | ReflectRef::Map(_)
            | ReflectRef::Scalar(_),
            ReflectRef::Array(_),
        )
        | (
            ReflectRef::Struct(_)
            | ReflectRef::TupleStruct(_)
            | ReflectRef::Tuple(_)
            | ReflectRef::Enum(_)
            | ReflectRef::Array(_)
            | ReflectRef::Map(_)
            | ReflectRef::Scalar(_),
            ReflectRef::List(_),
        )
        | (
            ReflectRef::Struct(_)
            | ReflectRef::TupleStruct(_)
            | ReflectRef::Tuple(_)
            | ReflectRef::Enum(_)
            | ReflectRef::Array(_)
            | ReflectRef::List(_)
            | ReflectRef::Scalar(_),
            ReflectRef::Map(_),
        )
        | (
            ReflectRef::Struct(_)
            | ReflectRef::TupleStruct(_)
            | ReflectRef::Tuple(_)
            | ReflectRef::Enum(_)
            | ReflectRef::Array(_)
            | ReflectRef::List(_)
            | ReflectRef::Map(_),
            ReflectRef::Scalar(_),
        )
        | (
            ReflectRef::TupleStruct(_)
            | ReflectRef::Tuple(_)
            | ReflectRef::Enum(_)
            | ReflectRef::Array(_)
            | ReflectRef::List(_)
            | ReflectRef::Map(_)
            | ReflectRef::Scalar(_),
            ReflectRef::Struct(_),
        ) => Some(false),
    }
}

fn reflect_eq_struct(a: &dyn Struct, b: &dyn Struct) -> Option<bool> {
    Some(
        a.fields_len() == b.fields_len() && {
            for (name, value_a) in a.fields() {
                let Some(value_b) = b.field(name) else {
                    return Some(false);
                };
                match reflect_eq(value_a, value_b) {
                    Some(true) => {}
                    Some(false) => {
                        return Some(false);
                    }
                    None => return None,
                }
            }
            true
        },
    )
}

fn reflect_eq_tuple_struct(a: &dyn TupleStruct, b: &dyn TupleStruct) -> Option<bool> {
    Some(
        a.fields_len() == b.fields_len() && {
            for (value_a, value_b) in a.fields().zip(b.fields()) {
                match reflect_eq(value_a, value_b) {
                    Some(true) => {}
                    Some(false) => {
                        return Some(false);
                    }
                    None => return None,
                }
            }
            true
        },
    )
}

fn reflect_eq_tuple(a: &dyn Tuple, b: &dyn Tuple) -> Option<bool> {
    Some(
        a.fields_len() == b.fields_len() && {
            for (value_a, value_b) in a.fields().zip(b.fields()) {
                match reflect_eq(value_a, value_b) {
                    Some(true) => {}
                    Some(false) => {
                        return Some(false);
                    }
                    None => return None,
                }
            }
            true
        },
    )
}

fn reflect_eq_enum(a: &dyn Enum, b: &dyn Enum) -> Option<bool> {
    Some(
        a.variant_name() == b.variant_name() && a.fields_len() == b.fields_len() && {
            match (a.variant_kind(), b.variant_kind()) {
                (VariantKind::Struct, VariantKind::Struct) => {
                    for field_a in a.fields() {
                        match field_a {
                            VariantField::Struct(name, value_a) => {
                                let Some(value_b) = b.field(name) else {
                                    return Some(false);
                                };
                                match reflect_eq(value_a, value_b) {
                                    Some(true) => {}
                                    Some(false) => {
                                        return Some(false);
                                    }
                                    None => return None,
                                }
                            }
                            VariantField::Tuple(_) => return Some(false),
                        }
                    }
                    true
                }

                (VariantKind::Tuple, VariantKind::Tuple) => {
                    for (field_a, field_b) in a.fields().zip(b.fields()) {
                        match (field_a, field_b) {
                            (VariantField::Tuple(value_a), VariantField::Tuple(value_b)) => {
                                match reflect_eq(value_a, value_b) {
                                    Some(true) => {}
                                    Some(false) => {
                                        return Some(false);
                                    }
                                    None => return None,
                                }
                            }
                            (
                                VariantField::Struct(_, _) | VariantField::Tuple(_),
                                VariantField::Struct(_, _),
                            )
                            | (VariantField::Struct(_, _), VariantField::Tuple(_)) => {
                                return Some(false);
                            }
                        }
                    }
                    true
                }

                (VariantKind::Unit, VariantKind::Unit) => {
                    return Some(true);
                }

                (VariantKind::Unit | VariantKind::Tuple, VariantKind::Struct)
                | (VariantKind::Unit | VariantKind::Struct, VariantKind::Tuple)
                | (VariantKind::Struct | VariantKind::Tuple, VariantKind::Unit) => {
                    return Some(false);
                }
            }
        },
    )
}

fn reflect_eq_array(a: &dyn Array, b: &dyn Array) -> Option<bool> {
    Some(
        a.len() == b.len() && {
            for (value_a, value_b) in a.iter().zip(b.iter()) {
                match reflect_eq(value_a, value_b) {
                    Some(true) => {}
                    Some(false) => {
                        return Some(false);
                    }
                    None => return None,
                }
            }
            true
        },
    )
}

fn reflect_eq_list(a: &dyn List, b: &dyn List) -> Option<bool> {
    Some(
        a.len() == b.len() && {
            for (value_a, value_b) in a.iter().zip(b.iter()) {
                match reflect_eq(value_a, value_b) {
                    Some(true) => {}
                    Some(false) => {
                        return Some(false);
                    }
                    None => return None,
                }
            }
            true
        },
    )
}

fn reflect_eq_map(a: &dyn Map, b: &dyn Map) -> Option<bool> {
    Some(
        a.len() == b.len() && {
            for (key, value_a) in a.iter() {
                let Some(value_b) = b.get(key) else {
                    return Some(false);
                };
                match reflect_eq(value_a, value_b) {
                    Some(true) => {}
                    Some(false) => {
                        return Some(false);
                    }
                    None => return None,
                }
            }
            true
        },
    )
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;

    use crate::{
        enum_::EnumValue, struct_::StructValue, tuple::TupleValue, tuple_struct::TupleStructValue,
    };

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn reflect_eq_scalar() {
        assert!(reflect_eq(&1_usize, &1_usize).unwrap());
        assert!(!reflect_eq(&1_usize, &2_usize).unwrap());

        assert!(reflect_eq(&1_u8, &1_u8).unwrap());
        assert!(!reflect_eq(&1_u8, &2_u8).unwrap());
        assert!(reflect_eq(&1_u16, &1_u16).unwrap());
        assert!(!reflect_eq(&1_u16, &2_u16).unwrap());
        assert!(reflect_eq(&1_u32, &1_u32).unwrap());
        assert!(!reflect_eq(&1_u32, &2_u32).unwrap());
        assert!(reflect_eq(&1_u64, &1_u64).unwrap());
        assert!(!reflect_eq(&1_u64, &2_u64).unwrap());
        assert!(reflect_eq(&1_u128, &1_u128).unwrap());
        assert!(!reflect_eq(&1_u128, &2_u128).unwrap());

        assert!(reflect_eq(&1_i8, &1_i8).unwrap());
        assert!(!reflect_eq(&1_i8, &2_i8).unwrap());
        assert!(reflect_eq(&1_i16, &1_i16).unwrap());
        assert!(!reflect_eq(&1_i16, &2_i16).unwrap());
        assert!(reflect_eq(&1_i32, &1_i32).unwrap());
        assert!(!reflect_eq(&1_i32, &2_i32).unwrap());
        assert!(reflect_eq(&1_i64, &1_i64).unwrap());
        assert!(!reflect_eq(&1_i64, &2_i64).unwrap());
        assert!(reflect_eq(&1_i128, &1_i128).unwrap());
        assert!(!reflect_eq(&1_i128, &2_i128).unwrap());

        assert!(reflect_eq(&true, &true).unwrap());
        assert!(reflect_eq(&false, &false).unwrap());
        assert!(!reflect_eq(&true, &false).unwrap());

        assert!(reflect_eq(&'a', &'a').unwrap());
        assert!(!reflect_eq(&'a', &'b').unwrap());

        assert!(reflect_eq(&1.0_f32, &1.0_f32).unwrap());
        assert!(!reflect_eq(&1.0_f32, &2.0_f32).unwrap());

        assert!(reflect_eq(&1.0_f64, &1.0_f64).unwrap());
        assert!(!reflect_eq(&1.0_f64, &2.0_f64).unwrap());

        assert!(reflect_eq(&String::from("a"), &String::from("a")).unwrap());
        assert!(!reflect_eq(&String::from("a"), &String::from("b")).unwrap());
    }

    #[test]
    fn reflect_eq_struct() {
        #[derive(Reflect, Debug, Clone)]
        #[reflect(crate_name(crate))]
        struct A {
            foo: i32,
            bar: bool,
        }

        assert!(reflect_eq(&A { foo: 1, bar: true }, &A { foo: 1, bar: true }).unwrap());
        assert!(!reflect_eq(&A { foo: 2, bar: true }, &A { foo: 1, bar: true }).unwrap());
        assert!(reflect_eq(
            &A { foo: 1, bar: true },
            &StructValue::new()
                .with_field("foo", 1)
                .with_field("bar", true)
        )
        .unwrap());
        assert!(reflect_eq(
            &A { foo: 1, bar: true },
            &StructValue::new()
                .with_field("bar", true)
                .with_field("foo", 1)
        )
        .unwrap());
        assert!(!reflect_eq(
            &A { foo: 1, bar: true },
            &StructValue::new()
                .with_field("foo", 1)
                .with_field("bar", true)
                .with_field("baz", 123.0)
        )
        .unwrap());
        assert!(!reflect_eq(
            &StructValue::new()
                .with_field("foo", 1)
                .with_field("bar", true)
                .with_field("baz", 123.0),
            &A { foo: 1, bar: true },
        )
        .unwrap());
        assert!(!reflect_eq(
            &StructValue::new()
                .with_field("foo", 1)
                .with_field("baz", 123.0),
            &A { foo: 1, bar: true },
        )
        .unwrap());
        assert!(!reflect_eq(
            &A { foo: 1, bar: true },
            &StructValue::new()
                .with_field("foo", 1)
                .with_field("baz", 123.0),
        )
        .unwrap());
    }

    #[test]
    fn reflect_eq_tuple_struct() {
        #[derive(Reflect, Debug, Clone)]
        #[reflect(crate_name(crate))]
        struct A(i32, bool);

        assert!(reflect_eq(&A(1, true), &A(1, true)).unwrap());
        assert!(!reflect_eq(&A(2, true), &A(1, true)).unwrap());
        assert!(reflect_eq(
            &A(1, true),
            &TupleStructValue::new().with_field(1).with_field(true)
        )
        .unwrap());
        // ordering does matter
        assert!(!reflect_eq(
            &A(1, true),
            &TupleStructValue::new().with_field(true).with_field(1)
        )
        .unwrap());
        assert!(!reflect_eq(
            &A(1, true),
            &TupleStructValue::new()
                .with_field(true)
                .with_field(1)
                .with_field(1)
        )
        .unwrap());
    }

    #[test]
    fn reflect_eq_tuple() {
        assert!(reflect_eq(&(1, true), &(1, true)).unwrap());
        assert!(!reflect_eq(&(2, true), &(1, true)).unwrap());
        assert!(reflect_eq(
            &(1, true),
            &TupleValue::new().with_field(1).with_field(true)
        )
        .unwrap());
        // ordering does matter
        assert!(!reflect_eq(
            &(1, true),
            &TupleValue::new().with_field(true).with_field(1)
        )
        .unwrap());
        assert!(!reflect_eq(
            &(1, true),
            &TupleValue::new()
                .with_field(true)
                .with_field(1)
                .with_field(1)
        )
        .unwrap());
    }

    #[test]
    fn reflect_eq_enum() {
        #[derive(Reflect, Debug, Clone)]
        #[reflect(crate_name(crate))]
        enum A {
            Struct { a: i32, b: bool },
            Tuple(i32, bool),
            Unit,
        }

        assert!(reflect_eq(&A::Struct { a: 1, b: true }, &A::Struct { a: 1, b: true }).unwrap());
        assert!(reflect_eq(
            &A::Struct { a: 1, b: true },
            &EnumValue::new_struct_variant("Struct")
                .with_struct_field("a", 1)
                .with_struct_field("b", true)
                .finish()
        )
        .unwrap());
        assert!(reflect_eq(
            &A::Struct { a: 1, b: true },
            &EnumValue::new_struct_variant("Struct")
                // field order doesn't matter
                .with_struct_field("b", true)
                .with_struct_field("a", 1)
                .finish()
        )
        .unwrap());
        assert!(!reflect_eq(
            &A::Struct { a: 1, b: true },
            // must have the same variant name
            &EnumValue::new_struct_variant("NotStruct")
                .with_struct_field("a", 1)
                .with_struct_field("b", true)
                .finish()
        )
        .unwrap());
        assert!(!reflect_eq(&A::Struct { a: 1, b: false }, &A::Struct { a: 1, b: true }).unwrap());
        assert!(!reflect_eq(&A::Struct { a: 1, b: false }, &A::Unit).unwrap());
        assert!(reflect_eq(&A::Tuple(1, true), &A::Tuple(1, true)).unwrap());
        assert!(!reflect_eq(&A::Tuple(1, true), &A::Tuple(1, false)).unwrap());
        assert!(reflect_eq(
            &A::Tuple(1, true),
            &EnumValue::new_tuple_variant("Tuple")
                .with_tuple_field(1)
                .with_tuple_field(true)
                .finish()
        )
        .unwrap());
        assert!(!reflect_eq(
            &A::Tuple(1, true),
            &EnumValue::new_tuple_variant("Tuple")
                .with_tuple_field(true)
                .with_tuple_field(1)
                .finish()
        )
        .unwrap());
        assert!(!reflect_eq(
            &A::Tuple(1, true),
            &EnumValue::new_tuple_variant("NotTuple")
                .with_tuple_field(1)
                .with_tuple_field(true)
                .finish()
        )
        .unwrap());
        assert!(reflect_eq(&A::Unit, &A::Unit).unwrap());
        assert!(reflect_eq(&A::Unit, &EnumValue::new_unit_variant("Unit")).unwrap());
        assert!(!reflect_eq(&A::Unit, &EnumValue::new_unit_variant("NotUnit")).unwrap());
    }

    #[test]
    fn reflect_eq_array() {
        assert!(reflect_eq(&[1, 2, 3], &[1, 2, 3]).unwrap());
        assert!(!reflect_eq(&[1, 2, 3], &[1, 2, 3, 4]).unwrap());
        assert!(!reflect_eq(&[1, 2, 3, 4], &[1, 2, 3]).unwrap());
        assert!(!reflect_eq(&[1, 2, 3], &[1, 2, 4]).unwrap());
    }

    #[test]
    fn reflect_eq_list() {
        assert!(reflect_eq(&vec![1, 2, 3], &vec![1, 2, 3]).unwrap());
        assert!(!reflect_eq(&vec![1, 2, 3], &vec![1, 2, 3, 4]).unwrap());
        assert!(!reflect_eq(&vec![1, 2, 3, 4], &vec![1, 2, 3]).unwrap());
        assert!(!reflect_eq(&vec![1, 2, 3], &vec![1, 2, 4]).unwrap());
    }

    #[test]
    fn reflect_eq_map() {
        assert!(reflect_eq(
            &BTreeMap::from([("a".to_owned(), 1), ("b".to_owned(), 2),]),
            &BTreeMap::from([("a".to_owned(), 1), ("b".to_owned(), 2),]),
        )
        .unwrap());
        assert!(reflect_eq(
            &BTreeMap::from([("a".to_owned(), 1), ("b".to_owned(), 2),]),
            &BTreeMap::from([("b".to_owned(), 2), ("a".to_owned(), 1),]),
        )
        .unwrap());
        assert!(!reflect_eq(
            &BTreeMap::from([("a".to_owned(), 2), ("b".to_owned(), 2),]),
            &BTreeMap::from([("b".to_owned(), 2), ("a".to_owned(), 1),]),
        )
        .unwrap());
        assert!(!reflect_eq(
            &BTreeMap::from([("a".to_owned(), 1), ("b".to_owned(), 2),]),
            &BTreeMap::from([("a".to_owned(), 1),]),
        )
        .unwrap());
    }
}
