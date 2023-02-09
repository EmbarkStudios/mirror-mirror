use crate::{
    type_info::{OpaqueType, Type, VariantField},
    Reflect, ScalarRef,
};

macro_rules! visit_scalar_fn {
    ($name:ident, $ty:ty) => {
        #[allow(clippy::ptr_arg)]
        fn $name(&mut self, value: $ty) -> Result<(), Self::Error> {
            Ok(())
        }
    };
}

#[allow(unused_variables)]
pub trait TryVisit {
    type Error;

    visit_scalar_fn!(try_visit_usize, usize);
    visit_scalar_fn!(try_visit_u8, u8);
    visit_scalar_fn!(try_visit_u16, u16);
    visit_scalar_fn!(try_visit_u32, u32);
    visit_scalar_fn!(try_visit_u64, u64);
    visit_scalar_fn!(try_visit_u128, u128);
    visit_scalar_fn!(try_visit_i8, i8);
    visit_scalar_fn!(try_visit_i16, i16);
    visit_scalar_fn!(try_visit_i32, i32);
    visit_scalar_fn!(try_visit_i64, i64);
    visit_scalar_fn!(try_visit_i128, i128);
    visit_scalar_fn!(try_visit_bool, bool);
    visit_scalar_fn!(try_visit_char, char);
    visit_scalar_fn!(try_visit_f32, f32);
    visit_scalar_fn!(try_visit_f64, f64);
    visit_scalar_fn!(try_visit_string, &String);

    fn try_visit_opaque(
        &mut self,
        value: &dyn Reflect,
        ty: OpaqueType<'_>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn try_visit<V>(visitor: &mut V, value: &dyn Reflect, ty: Type<'_>) -> Result<(), V::Error>
where
    V: TryVisit,
{
    match ty {
        Type::Scalar(_) => {
            let scalar = value.as_scalar().unwrap();
            match scalar {
                ScalarRef::usize(inner) => visitor.try_visit_usize(inner)?,
                ScalarRef::f32(inner) => visitor.try_visit_f32(inner)?,
                ScalarRef::bool(inner) => visitor.try_visit_bool(inner)?,
                ScalarRef::u8(inner) => visitor.try_visit_u8(inner)?,
                ScalarRef::u16(inner) => visitor.try_visit_u16(inner)?,
                ScalarRef::u32(inner) => visitor.try_visit_u32(inner)?,
                ScalarRef::u64(inner) => visitor.try_visit_u64(inner)?,
                ScalarRef::u128(inner) => visitor.try_visit_u128(inner)?,
                ScalarRef::i8(inner) => visitor.try_visit_i8(inner)?,
                ScalarRef::i16(inner) => visitor.try_visit_i16(inner)?,
                ScalarRef::i32(inner) => visitor.try_visit_i32(inner)?,
                ScalarRef::i64(inner) => visitor.try_visit_i64(inner)?,
                ScalarRef::i128(inner) => visitor.try_visit_i128(inner)?,
                ScalarRef::char(inner) => visitor.try_visit_char(inner)?,
                ScalarRef::f64(inner) => visitor.try_visit_f64(inner)?,
                ScalarRef::String(inner) => visitor.try_visit_string(inner)?,
            }
        }
        Type::Struct(struct_ty) => {
            let struct_ = value.as_struct().unwrap();

            for field_ty in struct_ty.field_types() {
                let field = struct_.field(field_ty.name()).unwrap();
                try_visit(visitor, field, field_ty.get_type())?;
            }
        }
        Type::TupleStruct(tuple_struct_ty) => {
            let tuple_struct = value.as_tuple_struct().unwrap();

            for (idx, field_ty) in tuple_struct_ty.field_types().enumerate() {
                let field = tuple_struct.field_at(idx).unwrap();
                try_visit(visitor, field, field_ty.get_type())?;
            }
        }
        Type::Tuple(tuple_ty) => {
            let tuple = value.as_tuple().unwrap();

            for (idx, field_ty) in tuple_ty.field_types().enumerate() {
                let field = tuple.field_at(idx).unwrap();
                try_visit(visitor, field, field_ty.get_type())?;
            }
        }
        Type::Enum(enum_ty) => {
            let enum_ = value.as_enum().unwrap();
            let variant_ty = enum_ty.variant(enum_.variant_name()).unwrap();

            for (idx, field_ty) in variant_ty.field_types().enumerate() {
                let field = match field_ty {
                    VariantField::Named(named_field_ty) => {
                        enum_.field(named_field_ty.name()).unwrap()
                    }
                    VariantField::Unnamed(_) => enum_.field_at(idx).unwrap(),
                };
                try_visit(visitor, field, field_ty.get_type())?;
            }
        }
        Type::List(list_ty) => {
            let list = value.as_list().unwrap();
            let element_ty = list_ty.element_type();

            for element in list.iter() {
                try_visit(visitor, element, element_ty)?;
            }
        }
        Type::Array(array_ty) => {
            let array = value.as_array().unwrap();
            let element_ty = array_ty.element_type();

            for element in array.iter() {
                try_visit(visitor, element, element_ty)?;
            }
        }
        Type::Map(map_ty) => {
            let map = value.as_map().unwrap();
            let value_ty = map_ty.value_type();

            for (_, value) in map.iter() {
                try_visit(visitor, value, value_ty)?;
            }
        }
        Type::Opaque(opaque_ty) => {
            visitor.try_visit_opaque(value, opaque_ty)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DescribeType;
    use alloc::collections::BTreeMap;
    use core::convert::Infallible;

    #[derive(Debug, Clone, Reflect)]
    #[reflect(crate_name(crate))]
    struct Foo {
        a: String,
        b: i32,
        c: Vec<Bar>,
    }

    #[derive(Debug, Clone, Reflect)]
    #[reflect(crate_name(crate))]
    enum Bar {
        A(BTreeMap<i32, i32>),
    }

    #[test]
    fn works() {
        let foo = Foo {
            a: "a".to_owned(),
            b: 1337,
            c: Vec::from([Bar::A(BTreeMap::from_iter([(1, 1), (2, 2)]))]),
        };

        #[derive(Default, Debug)]
        struct Visitor {
            string_count: usize,
            i32_count: usize,
        }

        impl TryVisit for Visitor {
            type Error = Infallible;

            fn try_visit_string(&mut self, _value: &String) -> Result<(), Self::Error> {
                self.string_count += 1;
                Ok(())
            }

            fn try_visit_i32(&mut self, _value: i32) -> Result<(), Self::Error> {
                self.i32_count += 1;
                Ok(())
            }
        }

        let mut visitor = Visitor::default();
        try_visit(
            &mut visitor,
            &foo,
            <Foo as DescribeType>::type_descriptor().get_type(),
        )
        .unwrap();

        assert_eq!(visitor.string_count, 1);
        assert_eq!(visitor.i32_count, 3);
    }
}
