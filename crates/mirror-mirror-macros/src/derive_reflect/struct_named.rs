use crate::stringify;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Field, FieldsNamed, Ident, Token};

type Fields = Punctuated<Field, Token![,]>;

pub(crate) fn expand(ident: &Ident, fields: FieldsNamed) -> syn::Result<TokenStream> {
    let fields = fields.named;

    let reflect = expand_reflect(ident, &fields);
    let from_reflect = expand_from_reflect(ident, &fields);
    let struct_ = expand_struct(ident, &fields);

    Ok(quote! {
        const _: () = {
            use mirror_mirror::*;
            use mirror_mirror::__private::*;

            #reflect
            #from_reflect
            #struct_

            impl From<#ident> for Value {
                fn from(data: #ident) -> Value {
                    data.to_value()
                }
            }

            fn trait_bounds()
            where
                #ident: std::clone::Clone + std::fmt::Debug,
            {}
        };
    })
}

fn expand_reflect(ident: &Ident, fields: &Fields) -> TokenStream {
    let fn_patch = {
        let code_for_fields = fields.iter().map(|field| {
            let field = stringify(&field.ident);
            quote! {
                if let Some(field) = value.field(#field) {
                    self.field_mut(#field).unwrap().patch(field);
                }
            }
        });

        quote! {
            fn patch(&mut self, value: &dyn Reflect) {
                if let Some(value) = value.as_struct() {
                    #(#code_for_fields)*
                }
            }
        }
    };

    let fn_to_value = {
        let code_for_fields = fields.iter().map(|field| {
            let ident = &field.ident;
            let field = stringify(ident);
            quote! {
                let value = value.with_field(#field, self.#ident.clone());
            }
        });

        quote! {
            fn to_value(&self) -> Value {
                let value = StructValue::default();
                #(#code_for_fields)*
                value.into()
            }
        }
    };

    quote! {
        impl Reflect for #ident {
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

            #fn_patch

            #fn_to_value

            fn clone_reflect(&self) -> Box<dyn Reflect> {
                Box::new(self.clone())
            }

            fn as_tuple(&self) -> Option<&dyn Tuple> {
                None
            }

            fn as_tuple_mut(&mut self) -> Option<&mut dyn Tuple> {
                None
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

            fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if f.alternate() {
                    write!(f, "{:#?}", self)
                } else {
                    write!(f, "{:?}", self)
                }
            }
        }
    }
}

fn expand_from_reflect(ident: &Ident, fields: &Fields) -> TokenStream {
    let fn_from_reflect = {
        let code_for_fields = fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            let field = stringify(ident);
            quote! {
                #ident: struct_.field(#field)?.downcast_ref::<#ty>()?.to_owned(),
            }
        });

        quote! {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                let struct_ = reflect.as_struct()?;
                Some(Self {
                    #(#code_for_fields)*
                })
            }
        }
    };

    quote! {
        impl FromReflect for #ident {
            #fn_from_reflect
        }
    }
}

fn expand_struct(ident: &Ident, fields: &Fields) -> TokenStream {
    let fn_field = {
        let code_for_fields = fields.iter().map(|field| {
            let ident = &field.ident;
            let field = stringify(ident);
            quote! {
                if name == #field {
                    return Some(&self.#ident);
                }
            }
        });

        quote! {
            fn field(&self, name: &str) -> Option<&dyn Reflect> {
                #(#code_for_fields)*
                None
            }
        }
    };

    let fn_field_mut = {
        let code_for_fields = fields.iter().map(|field| {
            let ident = &field.ident;
            let field = stringify(ident);
            quote! {
                if name == #field {
                    return Some(&mut self.#ident);
                }
            }
        });

        quote! {
            fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
                #(#code_for_fields)*
                None
            }
        }
    };

    let fn_fields = {
        let code_for_fields = fields.iter().map(|field| {
            let ident = &field.ident;
            let field = stringify(ident);
            quote! {
                (#field, self.#ident.as_reflect()),
            }
        });

        quote! {
            fn fields(&self) -> PairIter<'_> {
                let iter = [#(#code_for_fields)*];
                PairIter::new(iter)
            }
        }
    };

    let fn_fields_mut = {
        let code_for_fields = fields.iter().map(|field| {
            let ident = &field.ident;
            let field = stringify(ident);
            quote! {
                (#field, self.#ident.as_reflect_mut()),
            }
        });

        quote! {
            fn fields_mut(&mut self) -> PairIterMut<'_> {
                let iter = [#(#code_for_fields)*];
                PairIterMut::new(iter)
            }
        }
    };

    quote! {
        impl Struct for #ident {
            #fn_field
            #fn_field_mut
            #fn_fields
            #fn_fields_mut
        }
    }
}
