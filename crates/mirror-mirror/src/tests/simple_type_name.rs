use alloc::collections::BTreeMap;

use crate::type_info::SimpleTypeName;

fn simple_type_name<T>() -> String {
    SimpleTypeName::new_from_type::<T>().to_string()
}

#[test]
fn works() {
    struct Foo<'a, const N: usize>(&'a ());

    assert_eq!(simple_type_name::<String>(), "String");
    assert_eq!(simple_type_name::<i32>(), "i32");
    assert_eq!(simple_type_name::<bool>(), "bool");
    assert_eq!(simple_type_name::<()>(), "()");
    assert_eq!(simple_type_name::<(i32,)>(), "(i32,)");
    assert_eq!(simple_type_name::<(i32, String)>(), "(i32, String)");
    assert_eq!(simple_type_name::<Vec<i32>>(), "Vec<i32>");
    assert_eq!(simple_type_name::<Vec<&()>>(), "Vec<&()>");
    assert_eq!(simple_type_name::<Vec<&mut ()>>(), "Vec<&mut ()>");
    assert_eq!(simple_type_name::<Vec<&'static ()>>(), "Vec<&()>");
    assert_eq!(simple_type_name::<Vec<&'static mut ()>>(), "Vec<&mut ()>");
    assert_eq!(simple_type_name::<Option<String>>(), "Option<String>");
    assert_eq!(
        simple_type_name::<BTreeMap<i32, String>>(),
        "BTreeMap<i32, String>"
    );
    assert_eq!(
        simple_type_name::<BTreeMap<Vec<(i32, Option<bool>)>, String>>(),
        "BTreeMap<Vec<(i32, Option<bool>)>, String>"
    );
    assert_eq!(simple_type_name::<[i32; 10]>(), "[i32; 10]");
    assert_eq!(
        simple_type_name::<[BTreeMap<i32, i32>; 10]>(),
        "[BTreeMap<i32, i32>; 10]"
    );
    // type names don't include lifetimes
    assert_eq!(simple_type_name::<Foo<'static, 10>>(), "Foo<10>");
}
