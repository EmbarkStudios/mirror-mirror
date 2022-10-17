use std::{any::Any, collections::HashMap, fmt};

use crate::{Enum, FromReflect, Reflect, Value, ValueInner};
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

pub trait Struct: Reflect {
    fn field(&self, name: &str) -> Option<&dyn Reflect>;
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;

    fn fields(&self) -> StructFieldsIter<'_>;
    fn fields_mut(&mut self) -> StructFieldsIterMut<'_>;
}

pub struct StructFieldsIter<'a> {
    iter: Box<dyn Iterator<Item = (&'a str, &'a dyn Reflect)> + 'a>,
}

impl<'a> StructFieldsIter<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for StructFieldsIter<'a> {
    type Item = (&'a str, &'a dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct StructFieldsIterMut<'a> {
    iter: Box<dyn Iterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a>,
}

impl<'a> StructFieldsIterMut<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for StructFieldsIterMut<'a> {
    type Item = (&'a str, &'a mut dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Default, Readable, Writable, Serialize, Deserialize, Debug, Clone)]
pub struct StructValue {
    fields: HashMap<String, Value>,
}

impl StructValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_field(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.fields.insert(name.into(), value.into());
        self
    }
}

impl Reflect for StructValue {
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

    fn as_struct(&self) -> Option<&dyn Struct> {
        Some(self)
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        Some(self)
    }

    fn as_enum(&self) -> Option<&dyn Enum> {
        None
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        None
    }

    fn patch(&mut self, value: &dyn Reflect) {
        if let Some(struct_) = value.as_struct() {
            for (name, value) in self.fields_mut() {
                if let Some(new_value) = struct_.field(name) {
                    value.patch(new_value);
                }
            }
        }
    }

    fn to_value(&self) -> Value {
        Value(ValueInner::StructValue(self.clone()))
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl Struct for StructValue {
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        Some(self.fields.get(name)?)
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        Some(self.fields.get_mut(name)?)
    }

    fn fields(&self) -> StructFieldsIter<'_> {
        let iter = self
            .fields
            .iter()
            .map(|(key, value)| (&**key, value.as_reflect()));
        StructFieldsIter::new(iter)
    }

    fn fields_mut(&mut self) -> StructFieldsIterMut<'_> {
        let iter = self
            .fields
            .iter_mut()
            .map(|(key, value)| (&**key, value.as_reflect_mut()));
        StructFieldsIterMut::new(iter)
    }
}

impl FromReflect for StructValue {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        let struct_ = reflect.as_struct()?;
        let this = struct_
            .fields()
            .fold(StructValue::default(), |builder, (name, value)| {
                builder.with_field(name, value.to_value())
            });
        Some(this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GetField;

    #[derive(Reflect, Default, Clone, Eq, PartialEq, Debug)]
    struct Foo {
        field: i32,
    }

    #[test]
    fn accessing_fields() {
        let foo = Foo { field: 42 };
        let struct_ = foo.as_struct().unwrap();

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

        let patch = StructValue::default().with_field("field", 1337);

        foo.patch(&patch);

        assert_eq!(foo.field, 1337);
    }

    #[test]
    fn patching_struct_value() {
        let mut value = StructValue::default().with_field("field", 42);
        let patch = StructValue::default().with_field("field", 1337);
        value.patch(&patch);

        assert_eq!(
            value.field("field").unwrap().downcast_ref::<i32>().unwrap(),
            &1337
        );
    }

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
        let struct_value = foo.to_value();

        let json = serde_json::to_string(&struct_value).unwrap();

        let struct_value = serde_json::from_str::<Value>(&json).unwrap();
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

    #[test]
    fn struct_value_from_reflect() {
        let value = StructValue::default().with_field("foo", 42);
        let reflect = value.as_reflect();

        let value = StructValue::from_reflect(reflect).unwrap();

        assert_eq!(
            value.field("foo").unwrap().downcast_ref::<i32>().unwrap(),
            &42,
        );
    }

    #[test]
    fn box_dyn_reflect_as_reflect() {
        let foo = Foo::default();
        let mut box_dyn_reflect = Box::new(foo) as Box<dyn Reflect>;

        assert_eq!(
            box_dyn_reflect
                .as_struct()
                .unwrap()
                .field("field")
                .unwrap()
                .downcast_ref::<i32>()
                .unwrap(),
            &0,
        );

        box_dyn_reflect.patch(&StructValue::default().with_field("field", 42));

        assert_eq!(
            box_dyn_reflect
                .as_struct()
                .unwrap()
                .field("field")
                .unwrap()
                .downcast_ref::<i32>()
                .unwrap(),
            &42,
        );

        let foo = Foo::from_reflect(&box_dyn_reflect).unwrap();
        assert_eq!(foo, Foo { field: 42 });
    }

    #[test]
    fn deeply_nested() {
        #[derive(Reflect, Clone, Debug)]
        struct Foo {
            bar: Bar,
        }

        #[derive(Reflect, Clone, Debug)]
        struct Bar {
            baz: Baz,
        }

        #[derive(Reflect, Clone, Debug)]
        struct Baz {
            qux: i32,
        }

        let foo = Foo {
            bar: Bar {
                baz: Baz { qux: 42 },
            },
        };

        let &forty_two = (|| {
            foo.get_field::<Bar>("bar")?
                .get_field::<Baz>("baz")?
                .get_field::<i32>("qux")
        })()
        .unwrap();

        assert_eq!(forty_two, 42);
    }
}
