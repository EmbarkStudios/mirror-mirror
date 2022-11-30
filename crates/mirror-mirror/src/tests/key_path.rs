use std::collections::BTreeMap;

use crate::key_path::*;
use crate::{self as mirror_mirror, key_path};
use mirror_mirror::Reflect;

#[test]
#[allow(clippy::bool_assert_comparison)]
fn works() {
    #[derive(Reflect, Clone, Debug)]
    struct A {
        a: i32,
        b: B,
        c: C,
        d: BTreeMap<String, u32>,
        e: Vec<f32>,
    }

    #[derive(Reflect, Clone, Debug)]
    struct B {
        c: bool,
    }

    #[derive(Reflect, Clone, Debug)]
    enum C {
        C { d: String },
    }

    let mut a = A {
        a: 42,
        b: B { c: true },
        c: C::C {
            d: "foo".to_owned(),
        },
        d: BTreeMap::from([("fourtytwo".to_owned(), 42)]),
        e: Vec::from([1.0, 2.0, 3.0]),
    };

    assert_eq!(a.at::<i32>(key_path!(.a)).unwrap(), &42);

    assert_eq!(a.at::<bool>(key_path!(.b.c)).unwrap(), &true);
    assert!(a.at::<B>(key_path!(.b.c).pop().unwrap()).is_some());

    assert_eq!(a.at::<String>(key_path!(.c.d)).unwrap(), &"foo");

    assert_eq!(a.at::<u32>(key_path!(.d.fourtytwo)).unwrap(), &42);

    assert_eq!(a.at::<f32>(key_path!(.e[0])).unwrap(), &1.0);
    assert_eq!(a.at::<f32>(key_path!(.e[1])).unwrap(), &2.0);
    assert_eq!(a.at::<f32>(key_path!(.e[2])).unwrap(), &3.0);
    assert!(a.at::<f32>(key_path!(.e[3])).is_none());

    assert_eq!(a.b.c, true);
    *a.at_mut::<bool>(key_path!(.b.c)).unwrap() = false;
    assert_eq!(a.b.c, false);
}
