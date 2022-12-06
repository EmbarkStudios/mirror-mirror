use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::key_path;
use crate::struct_::StructValue;
use crate::type_info::GetMeta;
use crate::type_info::GetTypedPath;
use crate::FromReflect;
use crate::GetField;
use crate::Reflect;
use crate::Struct;
use crate::Typed;
use crate::Value;

#[derive(Reflect, Default, Clone, Eq, PartialEq, Debug)]
#[reflect(crate_name(crate))]
struct Foo {
    field: i32,
}

#[test]
fn accessing_fields() {
    let foo = Foo { field: 42 };
    let struct_ = foo.reflect_ref().as_struct().unwrap();

    let value = struct_
        .field("field")
        .unwrap()
        .downcast_ref::<i32>()
        .unwrap();

    assert_eq!(*value, 42);

    let value: Value = struct_.to_value();
    assert_eq!(value.get_field::<i32>("field").unwrap(), &42);
    assert_eq!(value.get_field::<i32>("field".to_owned()).unwrap(), &42);
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
            .reflect_ref()
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
            .reflect_ref()
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
    #[reflect(crate_name(crate))]
    struct Foo {
        bar: Bar,
    }

    #[derive(Reflect, Clone, Debug)]
    #[reflect(crate_name(crate))]
    struct Bar {
        baz: Baz,
    }

    #[derive(Reflect, Clone, Debug)]
    #[reflect(crate_name(crate))]
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

#[test]
fn from_reflect_with_value() {
    #[derive(Debug, Clone, Reflect, Default)]
    #[reflect(crate_name(crate))]
    pub struct Foo {
        pub number: Number,
    }

    #[derive(Debug, Clone, Reflect, Default)]
    #[reflect(crate_name(crate))]
    pub enum Number {
        #[default]
        One,
        Two,
        Three,
    }

    let value = StructValue::new().with_field("number", Number::One);

    assert!(Foo::from_reflect(&value).is_some());
}

#[test]
fn accessing_docs_in_type_info() {
    /// Here are the docs.
    ///
    /// Foo bar.
    #[derive(Reflect, Clone, Debug)]
    #[reflect(crate_name(crate))]
    struct Foo {
        inner: Vec<BTreeMap<String, Vec<Option<Inner>>>>,
    }

    #[derive(Reflect, Clone, Debug)]
    #[reflect(crate_name(crate))]
    enum Inner {
        Variant {
            /// Bingo!
            field: String,
        },
    }

    let type_info = <Foo as Typed>::type_info();

    assert_eq!(
        type_info.type_().docs(),
        &[" Here are the docs.", "", " Foo bar."]
    );

    let variant_info = type_info
        .type_()
        .at_typed(key_path!(.inner[0].map_key[0]{Some}[0]{Variant}))
        .unwrap()
        .as_variant()
        .unwrap();
    let field = variant_info.fields().next().unwrap();
    assert_eq!(field.name().unwrap(), "field");
    assert_eq!(field.docs(), &[" Bingo!"]);
}
