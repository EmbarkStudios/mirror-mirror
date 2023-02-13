#![allow(missing_debug_implementations, clippy::todo)]

use core::fmt::{self, Write};

use super::*;

pub trait PrettyPrintRoot {
    fn pretty_print_root(&self) -> RootPrettyPrinter<'_, Self> {
        RootPrettyPrinter { ty: self }
    }

    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

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
            Type::Enum(_) => todo!(),
            Type::List(_) => todo!(),
            Type::Array(_) => todo!(),
            Type::Map(_) => todo!(),
            Type::Scalar(_) => todo!(),
            Type::Opaque(_) => todo!(),
        }
    }
}

fn simple_type_name_fmt(type_name: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(name) = SimpleTypeName::new(type_name) {
        write!(f, "{name}")
    } else {
        f.write_str(type_name)
    }
}

const IDENT: &str = "    ";

impl<'a> PrettyPrintRoot for StructType<'a> {
    fn pretty_root_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("struct ")?;
        simple_type_name_fmt(self.type_name(), f)?;
        f.write_str(" {")?;
        if self.fields_len() != 0 {
            f.write_char('\n')?;
            for field in self.field_types() {
                f.write_str(IDENT)?;
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

#[cfg(test)]
mod tests {
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
    fn tuple_struct() {
        #[derive(Reflect, Clone, Debug)]
        #[reflect(crate_name(crate))]
        struct Foo(String, Vec<i32>);

        let type_descriptor = <Foo as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(println_and_format!("{pp}"), r#"struct Foo(String, Vec<i32>)"#);
    }

    #[test]
    fn tuple() {
        let type_descriptor = <(String, Vec<i32>) as DescribeType>::type_descriptor();
        let pp = type_descriptor.pretty_print_root();

        assert_eq!(println_and_format!("{pp}"), r#"(String, Vec<i32>)"#);
    }
}
