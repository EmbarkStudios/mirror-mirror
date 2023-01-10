use core::{any::type_name, fmt};

use syn::{
    token::Mut, AngleBracketedGenericArguments, Expr, ExprLit, GenericArgument, Ident, Lit,
    LitBool, LitFloat, LitInt, Path, PathArguments, PathSegment, Type, TypeArray, TypePath,
    TypeReference, TypeTuple,
};

/// A writer for simplified type names.
///
/// By default type names are fully qualified which sometimes isn't desired. This type can be used
/// to print the types names in a simplified form:
///
/// ```
/// use core::any::type_name;
/// use mirror_mirror::type_info::SimpleTypeName;
///
/// // the default, fully qualified type name
/// assert_eq!(
///     type_name::<Option<String>>(),
///     "core::option::Option<alloc::string::String>",
/// );
///
/// // the simplified type name which is less verbose
/// assert_eq!(
///     SimpleTypeName::new_from_type::<Option<String>>().to_string(),
///     "Option<String>",
/// );
/// ```
///
/// # Not all types are supported
///
/// `SimpleTypeName` doesn't support printing absolutely all Rust types. It supports the most
/// common ones but exotic types like function pointers or trait objects are not supported. Such
/// types don't implement [`DescribeType`](super::DescribeType) anyway and are unlikely to show up when using
/// `mirror-mirror`.
///
/// Thus be careful when calling `to_string()` or using `format!()` as those will panic on
/// unsupported types:
///
/// ```should_panic
/// use mirror_mirror::type_info::SimpleTypeName;
///
/// format!("{}", SimpleTypeName::new_from_type::<fn() -> i32>());
/// ```
///
/// Instead use `write!` which allows handling the error
///
/// ```
/// use mirror_mirror::type_info::SimpleTypeName;
/// use core::fmt::Write;
///
/// let mut buf = String::new();
/// match write!(&mut buf, "{}", SimpleTypeName::new_from_type::<fn() -> i32>()) {
///     Ok(_) => {
///         // all good
///         # unreachable!();
///     }
///     Err(_) => {
///         // unsupported type
///         //
///         // instead just write the type we get directly from the compiler
///         buf.clear();
///         write!(&mut buf, "{}", core::any::type_name::<fn() -> i32>()).unwrap();
///         # assert_eq!(buf, "fn() -> i32");
///     }
/// }
/// ```
///
/// If you need a type that isn't supported feel free to submit a pull request!
#[allow(missing_debug_implementations)]
pub struct SimpleTypeName {
    ty: Type,
}

impl SimpleTypeName {
    pub fn new(type_name: &str) -> Option<Self> {
        let ty = syn::parse_str::<Type>(type_name).ok()?;
        Some(Self { ty })
    }

    pub fn new_from_type<T>() -> Self {
        Self::new(type_name::<T>()).expect("failed to parse type name")
    }
}

impl fmt::Display for SimpleTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        TypeWriter { f }.write(&self.ty)
    }
}

trait WriteAst<T> {
    fn write(&mut self, ty: &T) -> fmt::Result;
}

struct TypeWriter<'a, 'b> {
    f: &'a mut fmt::Formatter<'b>,
}

#[allow(clippy::wildcard_in_or_patterns)]
impl<'a, 'b> WriteAst<Type> for TypeWriter<'a, 'b> {
    fn write(&mut self, ty: &Type) -> fmt::Result {
        match ty {
            Type::Array(inner) => self.write(inner),
            Type::Path(inner) => self.write(inner),
            Type::Tuple(inner) => self.write(inner),
            Type::Reference(inner) => self.write(inner),
            Type::BareFn(_)
            | Type::Group(_)
            | Type::ImplTrait(_)
            | Type::Infer(_)
            | Type::Macro(_)
            | Type::Never(_)
            | Type::Paren(_)
            | Type::Ptr(_)
            | Type::Slice(_)
            | Type::TraitObject(_)
            | Type::Verbatim(_)
            | _ => Err(fmt::Error),
        }
    }
}

impl<'a, 'b> WriteAst<TypeArray> for TypeWriter<'a, 'b> {
    fn write(&mut self, array: &TypeArray) -> fmt::Result {
        let TypeArray {
            bracket_token: _,
            elem,
            semi_token: _,
            len,
        } = array;
        write!(self.f, "[")?;
        self.write(&**elem)?;
        write!(self.f, "; ")?;
        self.write(len)?;
        write!(self.f, "]")?;
        Ok(())
    }
}

#[allow(clippy::wildcard_in_or_patterns)]
impl<'a, 'b> WriteAst<Expr> for TypeWriter<'a, 'b> {
    fn write(&mut self, expr: &Expr) -> fmt::Result {
        match expr {
            Expr::Lit(inner) => self.write(inner),
            Expr::Array(_)
            | Expr::Assign(_)
            | Expr::AssignOp(_)
            | Expr::Async(_)
            | Expr::Await(_)
            | Expr::Binary(_)
            | Expr::Block(_)
            | Expr::Box(_)
            | Expr::Break(_)
            | Expr::Call(_)
            | Expr::Cast(_)
            | Expr::Closure(_)
            | Expr::Continue(_)
            | Expr::Field(_)
            | Expr::ForLoop(_)
            | Expr::Group(_)
            | Expr::If(_)
            | Expr::Index(_)
            | Expr::Let(_)
            | Expr::Loop(_)
            | Expr::Macro(_)
            | Expr::Match(_)
            | Expr::MethodCall(_)
            | Expr::Paren(_)
            | Expr::Path(_)
            | Expr::Range(_)
            | Expr::Reference(_)
            | Expr::Repeat(_)
            | Expr::Return(_)
            | Expr::Struct(_)
            | Expr::Try(_)
            | Expr::TryBlock(_)
            | Expr::Tuple(_)
            | Expr::Type(_)
            | Expr::Unary(_)
            | Expr::Unsafe(_)
            | Expr::Verbatim(_)
            | Expr::While(_)
            | Expr::Yield(_)
            | _ => Err(fmt::Error),
        }
    }
}

impl<'a, 'b> WriteAst<ExprLit> for TypeWriter<'a, 'b> {
    fn write(&mut self, lit: &ExprLit) -> fmt::Result {
        let ExprLit { attrs: _, lit } = lit;
        self.write(lit)?;
        Ok(())
    }
}

impl<'a, 'b> WriteAst<Lit> for TypeWriter<'a, 'b> {
    fn write(&mut self, lit: &Lit) -> fmt::Result {
        match lit {
            Lit::Int(inner) => self.write(inner),
            Lit::Bool(inner) => self.write(inner),
            Lit::Float(inner) => self.write(inner),
            Lit::Str(_) | Lit::ByteStr(_) | Lit::Byte(_) | Lit::Char(_) | Lit::Verbatim(_) => {
                Err(fmt::Error)
            }
        }
    }
}

impl<'a, 'b> WriteAst<LitInt> for TypeWriter<'a, 'b> {
    fn write(&mut self, lit: &LitInt) -> fmt::Result {
        write!(self.f, "{lit}")
    }
}

impl<'a, 'b> WriteAst<LitFloat> for TypeWriter<'a, 'b> {
    fn write(&mut self, lit: &LitFloat) -> fmt::Result {
        write!(self.f, "{lit}")
    }
}

impl<'a, 'b> WriteAst<LitBool> for TypeWriter<'a, 'b> {
    fn write(&mut self, lit: &LitBool) -> fmt::Result {
        write!(self.f, "{}", lit.value)
    }
}

impl<'a, 'b> WriteAst<TypePath> for TypeWriter<'a, 'b> {
    fn write(&mut self, ty: &TypePath) -> fmt::Result {
        let TypePath { qself, path } = ty;
        if qself.is_some() {
            return Err(fmt::Error);
        }
        self.write(path)?;
        Ok(())
    }
}

impl<'a, 'b> WriteAst<Path> for TypeWriter<'a, 'b> {
    fn write(&mut self, path: &Path) -> fmt::Result {
        let Path {
            leading_colon: _,
            segments,
        } = path;
        let last = segments.last().ok_or(fmt::Error)?;
        self.write(last)?;
        Ok(())
    }
}

impl<'a, 'b> WriteAst<PathSegment> for TypeWriter<'a, 'b> {
    fn write(&mut self, path_segment: &PathSegment) -> fmt::Result {
        let PathSegment { ident, arguments } = path_segment;
        self.write(ident)?;
        self.write(arguments)?;
        Ok(())
    }
}

impl<'a, 'b> WriteAst<Ident> for TypeWriter<'a, 'b> {
    fn write(&mut self, ident: &Ident) -> fmt::Result {
        write!(self.f, "{ident}")?;
        Ok(())
    }
}

impl<'a, 'b> WriteAst<PathArguments> for TypeWriter<'a, 'b> {
    fn write(&mut self, args: &PathArguments) -> fmt::Result {
        match args {
            PathArguments::None => Ok(()),
            PathArguments::AngleBracketed(inner) => self.write(inner),
            PathArguments::Parenthesized(_) => Err(fmt::Error),
        }
    }
}

impl<'a, 'b> WriteAst<AngleBracketedGenericArguments> for TypeWriter<'a, 'b> {
    fn write(&mut self, args: &AngleBracketedGenericArguments) -> fmt::Result {
        let AngleBracketedGenericArguments {
            colon2_token: _,
            lt_token: _,
            args,
            gt_token: _,
        } = args;
        write!(self.f, "<")?;
        let mut args = args.iter().peekable();
        while let Some(arg) = args.next() {
            self.write(arg)?;
            if args.peek().is_some() {
                write!(self.f, ", ")?;
            }
        }
        write!(self.f, ">")?;
        Ok(())
    }
}

impl<'a, 'b> WriteAst<GenericArgument> for TypeWriter<'a, 'b> {
    fn write(&mut self, arg: &GenericArgument) -> fmt::Result {
        match arg {
            GenericArgument::Type(inner) => self.write(inner),
            GenericArgument::Const(inner) => self.write(inner),
            GenericArgument::Lifetime(_)
            | GenericArgument::Binding(_)
            | GenericArgument::Constraint(_) => Err(fmt::Error),
        }
    }
}

impl<'a, 'b> WriteAst<TypeTuple> for TypeWriter<'a, 'b> {
    fn write(&mut self, ty: &TypeTuple) -> fmt::Result {
        let TypeTuple {
            paren_token: _,
            elems,
        } = ty;
        write!(self.f, "(")?;
        if elems.len() == 1 {
            for elem in elems {
                self.write(elem)?;
                write!(self.f, ",")?;
            }
        } else {
            let mut elems = elems.iter().peekable();
            while let Some(elem) = elems.next() {
                self.write(elem)?;
                if elems.peek().is_some() {
                    write!(self.f, ", ")?;
                }
            }
        }
        write!(self.f, ")")?;
        Ok(())
    }
}

impl<'a, 'b> WriteAst<TypeReference> for TypeWriter<'a, 'b> {
    fn write(&mut self, ty_ref: &TypeReference) -> fmt::Result {
        let TypeReference {
            and_token: _,
            // type names don't include lifetimes
            lifetime: _,
            mutability,
            elem,
        } = ty_ref;
        write!(self.f, "&")?;
        if let Some(mutability) = mutability {
            self.write(mutability)?;
            write!(self.f, " ")?;
        }
        self.write(&**elem)?;
        Ok(())
    }
}

impl<'a, 'b> WriteAst<Mut> for TypeWriter<'a, 'b> {
    fn write(&mut self, _: &Mut) -> fmt::Result {
        write!(self.f, "mut")
    }
}
