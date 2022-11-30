use std::collections::BTreeMap;

use crate::key_path;
use crate::key_path::*;
use crate::{self as mirror_mirror};
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

    assert!(a.at(key_path!()).unwrap().downcast_ref::<A>().is_some());

    assert_eq!(
        a.at(key_path!(.a)).unwrap().downcast_ref::<i32>().unwrap(),
        &42
    );

    assert_eq!(
        a.at(key_path!(.b.c))
            .unwrap()
            .downcast_ref::<bool>()
            .unwrap(),
        &true
    );

    assert_eq!(
        a.at(key_path!(.c.d))
            .unwrap()
            .downcast_ref::<String>()
            .unwrap(),
        &"foo"
    );

    assert_eq!(
        a.at(key_path!(.d.fourtytwo))
            .unwrap()
            .downcast_ref::<u32>()
            .unwrap(),
        &42
    );

    assert_eq!(
        a.at(key_path!(.e[0]))
            .unwrap()
            .downcast_ref::<f32>()
            .unwrap(),
        &1.0
    );
    assert_eq!(
        a.at(key_path!(.e[1]))
            .unwrap()
            .downcast_ref::<f32>()
            .unwrap(),
        &2.0
    );
    assert_eq!(
        a.at(key_path!(.e[2]))
            .unwrap()
            .downcast_ref::<f32>()
            .unwrap(),
        &3.0
    );
    assert!(a.at(key_path!(.e[3])).is_none());

    assert_eq!(a.b.c, true);
    *a.at_mut(key_path!(.b.c)).unwrap().downcast_mut().unwrap() = false;
    assert_eq!(a.b.c, false);
}

#[test]
fn display() {
    assert_eq!(
        key_path!(.a.b.c[1][2].d[3]).to_string(),
        ".a.b.c[1][2].d[3]"
    );
}
