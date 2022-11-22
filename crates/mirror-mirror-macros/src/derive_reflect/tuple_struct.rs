use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, spanned::Spanned, Field, FieldsUnnamed, Ident, Index, Token};

type Fields = Punctuated<Field, Token![,]>;

pub(crate) fn expand(ident: &Ident, fields: FieldsUnnamed) -> syn::Result<TokenStream> {
    let fields = fields.unnamed;

    let reflect = expand_reflect(ident, &fields);
    let from_reflect = expand_from_reflect(ident, &fields);
    let tuple_struct = expand_tuple_struct(ident, &fields);

    Ok(quote! {
        #reflect
        #from_reflect
        #tuple_struct

        impl From<#ident> for Value {
            fn from(data: #ident) -> Value {
                data.to_value()
            }
        }

        fn trait_bounds()
        where
            #ident: std::clone::Clone + std::fmt::Debug,
        {}
    })
}

fn expand_reflect(ident: &Ident, fields: &Fields) -> TokenStream {
    let fn_patch = {
        quote! {
            fn patch(&mut self, value: &dyn Reflect) {
                if let Some(value) = value.as_tuple_struct() {
                    for (current, new) in self.elements_mut().zip(value.elements()) {
                        current.patch(new);
                    }
                }
            }
        }
    };

    let fn_to_value = {
        let code_for_fields = fields.iter().enumerate().map(|(idx, field)| {
            let field_index = Index {
                index: idx as u32,
                span: field.span(),
            };
            quote! {
                let value = value.with_element(self.#field_index.clone());
            }
        });

        quote! {
            fn to_value(&self) -> Value {
                let value = TupleStructValue::default();
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
                None
            }

            fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
                None
            }

            fn as_tuple_struct(&self) -> Option<&dyn TupleStruct> {
                Some(self)
            }

            fn as_tuple_struct_mut(&mut self) -> Option<&mut dyn TupleStruct> {
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
        let code_for_fields = fields.iter().enumerate().map(|(idx, field)| {
            let field_index = Index {
                index: idx as u32,
                span: field.span(),
            };
            let ty = &field.ty;
            quote! {
                #field_index: tuple_struct
                    .element(#field_index)?
                    .downcast_ref::<#ty>()?
                    .to_owned(),
            }
        });

        quote! {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                let tuple_struct = reflect.as_tuple_struct()?;
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

fn expand_tuple_struct(ident: &Ident, fields: &Fields) -> TokenStream {
    let fn_element = {
        let match_arms = fields.iter().enumerate().map(|(idx, field)| {
            let field_index = Index {
                index: idx as u32,
                span: field.span(),
            };
            quote! {
                #idx => Some(self.#field_index.as_reflect()),
            }
        });

        quote! {
            fn element(&self, index: usize) -> Option<&dyn Reflect> {
                match index {
                    #(#match_arms)*
                    _ => None,
                }
            }
        }
    };

    let fn_element_mut = {
        let match_arms = fields.iter().enumerate().map(|(idx, field)| {
            let field_index = Index {
                index: idx as u32,
                span: field.span(),
            };
            quote! {
                #idx => Some(self.#field_index.as_reflect_mut()),
            }
        });

        quote! {
            fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
                match index {
                    #(#match_arms)*
                    _ => None,
                }
            }
        }
    };

    let fn_elements = {
        let code_for_fields = fields.iter().enumerate().map(|(idx, field)| {
            let field_index = Index {
                index: idx as u32,
                span: field.span(),
            };
            quote! {
                self.#field_index.as_reflect(),
            }
        });

        quote! {
            fn elements(&self) -> ValueIter<'_> {
                let iter = [#(#code_for_fields)*];
                ValueIter::new(iter)
            }
        }
    };

    let fn_elements_mut = {
        let code_for_fields = fields.iter().enumerate().map(|(idx, field)| {
            let field_index = Index {
                index: idx as u32,
                span: field.span(),
            };
            quote! {
                self.#field_index.as_reflect_mut(),
            }
        });

        quote! {
            fn elements_mut(&mut self) -> ValueIterMut<'_> {
                let iter = [#(#code_for_fields)*];
                ValueIterMut::new(iter)
            }
        }
    };

    quote! {
        impl TupleStruct for #ident {
            #fn_element
            #fn_element_mut
            #fn_elements
            #fn_elements_mut
        }
    }
}
