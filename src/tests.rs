use std::any::Any;

use crate::{FieldsIter, FieldsIterMut, FromReflect, Reflect, Struct, StructValue};

#[derive(Default)]
struct Foo {
    field: i32,
}

impl Reflect for Foo {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(value) = value.as_struct() {
            if let Some(field) = value.field("field") {
                self.field_mut("field").unwrap().patch(field);
            }
        }
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        Some(self)
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        Some(self)
    }
}

impl FromReflect for Foo {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let struct_ = reflect.as_struct()?;
        Some(Self {
            field: struct_.field("field")?.downcast_ref::<i32>()?.to_owned(),
        })
    }
}

impl Struct for Foo {
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        if name == "field" {
            return Some(&self.field);
        }

        None
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        if name == "field" {
            return Some(&mut self.field);
        }

        None
    }

    fn into_value(self) -> StructValue {
        StructValue::builder().set("field", self.field).build()
    }

    fn fields(&self) -> FieldsIter<'_> {
        let iter = std::iter::once(("field", self.field.as_reflect()));
        FieldsIter::new(iter)
    }

    fn fields_mut(&mut self) -> FieldsIterMut<'_> {
        let iter = std::iter::once(("field", self.field.as_reflect_mut()));
        FieldsIterMut::new(iter)
    }
}

#[test]
fn accessing_fields() {
    let foo = Foo { field: 42 };
    let struct_: &dyn Struct = foo.as_struct().unwrap();

    let value = struct_
        .field("field")
        .unwrap()
        .downcast_ref::<i32>()
        .unwrap();

    assert_eq!(*value, 42);
}

#[test]
fn patching() {
    let mut foo = Foo { field: 42 };

    let patch = StructValue::builder().set("field", 1337).build();

    foo.patch(&patch);

    assert_eq!(foo.field, 1337);
}

#[test]
fn patching_struct_value() {
    let mut value = StructValue::builder().set("field", 42).build();
    let patch = StructValue::builder().set("field", 1337).build();
    value.patch(&patch);

    assert_eq!(
        value.field("field").unwrap().downcast_ref::<i32>().unwrap(),
        &1337
    );
}

// #[test]
// fn patching_value() {
//     todo!()
// }

#[test]
fn from_reflect() {
    let foo = Foo::default();
    let foo_reflect: &dyn Reflect = &foo;

    let foo = Foo::from_reflect(foo_reflect).unwrap();

    assert_eq!(foo.field, 0);
}

#[test]
fn serialize_deserialize() {
    let foo = Foo::default();
    let struct_value = foo.into_value();

    let json = serde_json::to_string(&struct_value).unwrap();

    let struct_value = serde_json::from_str::<StructValue>(&json).unwrap();
    let foo = Foo::from_reflect(&struct_value).unwrap();

    assert_eq!(foo.field, 0);
}

#[test]
fn fields() {
    let foo = Foo::default();

    for (name, value) in foo.fields() {
        if name == "field" {
            assert_eq!(foo.field, i32::from_reflect(value).unwrap());
        } else {
            panic!("Unknown field {name:?}");
        }
    }
}
