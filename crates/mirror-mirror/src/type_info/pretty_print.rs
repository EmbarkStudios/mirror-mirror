use core::fmt::{self, Write};

use super::*;

pub trait PrettyPrintRoot: super::private::Sealed {
    fn pretty_print_root(&self) -> RootPrettyPrinter<'_, Self> {
        RootPrettyPrinter { ty: self }
    }

    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

#[derive(Debug)]
pub struct RootPrettyPrinter<'a, T>
where
    T: ?Sized,
{
    ty: &'a T,
}

impl<'a, T> fmt::Display for RootPrettyPrinter<'a, T>
where
    T: PrettyPrintRoot,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ty.pretty_root_fmt(f)
    }
}

impl PrettyPrintRoot for TypeDescriptor {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get_type().pretty_root_fmt(f)
    }
}

impl<'a> PrettyPrintRoot for Type<'a> {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Struct(inner) => inner.pretty_root_fmt(f),
            Type::TupleStruct(inner) => inner.pretty_root_fmt(f),
            Type::Tuple(inner) => inner.pretty_root_fmt(f),
            Type::Enum(inner) => inner.pretty_root_fmt(f),
            Type::List(inner) => inner.pretty_root_fmt(f),
            Type::Array(inner) => inner.pretty_root_fmt(f),
            Type::Map(inner) => inner.pretty_root_fmt(f),
            Type::Scalar(inner) => inner.pretty_root_fmt(f),
            Type::Opaque(inner) => inner.pretty_root_fmt(f),
        }
    }
}

#[cfg(feature = "std")]
fn simple_type_name_fmt(type_name: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(name) = super::SimpleTypeName::new(type_name) {
        write!(f, "{name}")
    } else {
        f.write_str(type_name)
    }
}

#[cfg(not(feature = "std"))]
fn simple_type_name_fmt(type_name: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(type_name)
}

const TAB: &str = "    ";

impl<'a> PrettyPrintRoot for StructType<'a> {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("struct ")?;
        simple_type_name_fmt(self.type_name(), f)?;
        f.write_str(" {")?;
        if self.fields_len() != 0 {
            f.write_char('\n')?;
            for field in self.field_types() {
                f.write_str(TAB)?;
                f.write_str(field.name())?;
                f.write_str(": ")?;
                simple_type_name_fmt(field.get_type().type_name(), f)?;
                f.write_str(",")?;
                f.write_char('\n')?;
            }
        }
        f.write_str("}")?;
        Ok(())
    }
}

impl<'a> PrettyPrintRoot for TupleStructType<'a> {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("struct ")?;
        simple_type_name_fmt(self.type_name(), f)?;
        f.write_str("(")?;
        let mut fields = self.field_types().peekable();
        while let Some(field) = fields.next() {
            simple_type_name_fmt(field.get_type().type_name(), f)?;
            if fields.peek().is_some() {
                f.write_str(", ")?;
            }
        }
        f.write_str(")")?;
        Ok(())
    }
}

impl<'a> PrettyPrintRoot for TupleType<'a> {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("(")?;
        let mut fields = self.field_types().peekable();
        while let Some(field) = fields.next() {
            simple_type_name_fmt(field.get_type().type_name(), f)?;
            if fields.peek().is_some() {
                f.write_str(", ")?;
            }
        }
        f.write_str(")")?;
        Ok(())
    }
}

impl<'a> PrettyPrintRoot for EnumType<'a> {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("enum ")?;
        simple_type_name_fmt(self.type_name(), f)?;
        f.write_str(" {")?;
        if self.variants_len() != 0 {
            f.write_char('\n')?;
            for variant in self.variants() {
                match variant {
                    Variant::Struct(struct_variant) => {
                        f.write_str(TAB)?;
                        f.write_str(struct_variant.name())?;
                        f.write_str(" {")?;
                        if struct_variant.fields_len() != 0 {
                            f.write_char('\n')?;
                            for field in struct_variant.field_types() {
                                f.write_str(TAB)?;
                                f.write_str(TAB)?;
                                f.write_str(field.name())?;
                                f.write_str(": ")?;
                                simple_type_name_fmt(field.get_type().type_name(), f)?;
                                f.write_str(",")?;
                                f.write_char('\n')?;
                            }
                            f.write_str(TAB)?;
                        }
                        f.write_str("},\n")?;
                    }
                    Variant::Tuple(tuple_variant) => {
                        f.write_str(TAB)?;
                        f.write_str(tuple_variant.name())?;
                        f.write_str("(")?;
                        let mut fields = tuple_variant.field_types().peekable();
                        while let Some(field) = fields.next() {
                            simple_type_name_fmt(field.get_type().type_name(), f)?;
                            if fields.peek().is_some() {
                                f.write_str(", ")?;
                            }
                        }
                        f.write_str("),\n")?;
                    }
                    Variant::Unit(unit_variant) => {
                        f.write_str(TAB)?;
                        f.write_str(unit_variant.name())?;
                        f.write_str(",\n")?;
                    }
                }
            }
        }
        f.write_str("}")?;
        Ok(())
    }
}

impl<'a> PrettyPrintRoot for ListType<'a> {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('[')?;
        simple_type_name_fmt(self.element_type().type_name(), f)?;
        f.write_char(']')?;
        Ok(())
    }
}

impl<'a> PrettyPrintRoot for ArrayType<'a> {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('[')?;
        simple_type_name_fmt(self.element_type().type_name(), f)?;
        f.write_str("; ")?;
        write!(f, "{}", self.len())?;
        f.write_char(']')?;
        Ok(())
    }
}

impl<'a> PrettyPrintRoot for MapType<'a> {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('[')?;
        simple_type_name_fmt(self.key_type().type_name(), f)?;
        f.write_str(": ")?;
        simple_type_name_fmt(self.value_type().type_name(), f)?;
        f.write_char(']')?;
        Ok(())
    }
}

impl PrettyPrintRoot for ScalarType {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScalarType::usize => f.write_str("usize")?,
            ScalarType::u8 => f.write_str("u8")?,
            ScalarType::u16 => f.write_str("u16")?,
            ScalarType::u32 => f.write_str("u32")?,
            ScalarType::u64 => f.write_str("u64")?,
            ScalarType::u128 => f.write_str("u128")?,
            ScalarType::i8 => f.write_str("i8")?,
            ScalarType::i16 => f.write_str("i16")?,
            ScalarType::i32 => f.write_str("i32")?,
            ScalarType::i64 => f.write_str("i64")?,
            ScalarType::i128 => f.write_str("i128")?,
            ScalarType::bool => f.write_str("bool")?,
            ScalarType::char => f.write_str("char")?,
            ScalarType::f32 => f.write_str("f32")?,
            ScalarType::f64 => f.write_str("f64")?,
            ScalarType::String => f.write_str("String")?,
        }
        Ok(())
    }
}

impl<'a> PrettyPrintRoot for OpaqueType<'a> {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        simple_type_name_fmt(self.type_name(), f)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;
    use crate::{DescribeType, Reflect};

    // makes it a little easier to see what the output was if a test fails
    macro_rules! println_and_format {
        ($($tt:tt)*) => {
            {
                println!($($tt)*);
                format!($($tt)*)
            }
        };
    }

    #[test]
    fn struct_() {
        #[derive(Reflect, Clone, Debug)]
        #[reflect(crate_name(crate))]
        struct Foo {
            a: String,
            b: Vec<i32>,
        }

        let type_descriptor = <Foo as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(
            println_and_format!("{pp}"),
            r#"struct Foo {
    a: String,
    b: Vec<i32>,
}"#
        );
    }

    #[test]
    fn struct_empty() {
        #[derive(Reflect, Clone, Debug)]
        #[reflect(crate_name(crate))]
        struct Foo {}

        let type_descriptor = <Foo as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(println_and_format!("{pp}"), r#"struct Foo {}"#);
    }

    #[test]
    fn tuple_struct() {
        #[derive(Reflect, Clone, Debug)]
        #[reflect(crate_name(crate))]
        struct Foo(String, Vec<i32>);

        let type_descriptor = <Foo as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(
            println_and_format!("{pp}"),
            r#"struct Foo(String, Vec<i32>)"#
        );
    }

    #[test]
    fn tuple_struct_empty() {
        #[derive(Reflect, Clone, Debug)]
        #[reflect(crate_name(crate))]
        struct Foo();

        let type_descriptor = <Foo as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(println_and_format!("{pp}"), r#"struct Foo()"#);
    }

    #[test]
    fn unit_struct() {
        #[derive(Reflect, Clone, Debug)]
        #[reflect(crate_name(crate))]
        struct Foo;

        let type_descriptor = <Foo as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        // unit structs are treated as empty structs
        assert_eq!(println_and_format!("{pp}"), r#"struct Foo {}"#);
    }

    #[test]
    fn tuple() {
        let type_descriptor = <(String, Vec<i32>) as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(println_and_format!("{pp}"), r#"(String, Vec<i32>)"#);
    }

    #[test]
    fn enum_() {
        #[derive(Reflect, Clone, Debug)]
        #[reflect(crate_name(crate))]
        enum Foo {
            A(String, i32),
            A2(),
            B { b: Vec<i32> },
            B2 {},
            C,
        }

        let type_descriptor = <Foo as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(
            println_and_format!("{pp}"),
            r#"enum Foo {
    A(String, i32),
    A2(),
    B {
        b: Vec<i32>,
    },
    B2 {},
    C,
}"#
        );
    }

    #[test]
    fn list() {
        let type_descriptor = <Vec<(String, i32)> as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(println_and_format!("{pp}"), r#"[(String, i32)]"#);
    }

    #[test]
    fn array() {
        let type_descriptor = <[(String, i32); 10] as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(println_and_format!("{pp}"), r#"[(String, i32); 10]"#);
    }

    #[test]
    fn map() {
        let type_descriptor = <BTreeMap<String, i32> as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(println_and_format!("{pp}"), r#"[String: i32]"#);
    }

    #[test]
    fn scalar() {
        let type_descriptor = <String as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(println_and_format!("{pp}"), r#"String"#);
    }

    #[test]
    fn opaque() {
        let type_descriptor = <Duration as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(println_and_format!("{pp}"), r#"Duration"#);
    }
}
