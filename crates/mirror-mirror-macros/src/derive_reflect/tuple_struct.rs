use super::attrs::AttrsDatabase;
use super::attrs::ItemAttrs;
use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Field;
use syn::FieldsUnnamed;
use syn::Ident;
use syn::Index;
use syn::Token;

type Fields = Punctuated<Field, Token![,]>;

pub(super) fn expand(
    ident: &Ident,
    fields: FieldsUnnamed,
    attrs: ItemAttrs,
) -> syn::Result<TokenStream> {
    let field_attrs = AttrsDatabase::new_from_unnamed(&fields)?;

    let fields = fields.unnamed;

    let reflect = expand_reflect(ident, &fields, attrs, &field_attrs);
    let from_reflect = expand_from_reflect(ident, &fields, &field_attrs);
    let tuple_struct = expand_tuple_struct(ident, &fields, &field_attrs);

    Ok(quote! {
        #reflect
        #from_reflect
        #tuple_struct
    })
}

fn expand_reflect(
    ident: &Ident,
    fields: &Fields,
    attrs: ItemAttrs,
    field_attrs: &AttrsDatabase<usize>,
) -> TokenStream {
    let fn_patch = {
        let code_for_fields = fields
            .iter()
            .enumerate()
            .filter(field_attrs.filter_out_skipped_unnamed())
            .map(|(idx, _)| {
                quote! {
                    if let Some(new_value) = tuple_struct.element(#idx) {
                        self.element_mut(#idx).unwrap().patch(new_value);
                    }
                }
            });

        quote! {
            fn patch(&mut self, value: &dyn Reflect) {
                if let Some(tuple_struct) = value.reflect_ref().as_tuple_struct() {
                    #(#code_for_fields)*
                }
            }
        }
    };

    let fn_to_value = {
        let code_for_fields = fields
            .iter()
            .enumerate()
            .filter(field_attrs.filter_out_skipped_unnamed())
            .map(|(idx, field)| {
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

    let fn_type_info = {
        let code_for_fields = fields
            .iter()
            .enumerate()
            .filter(field_attrs.filter_out_skipped_unnamed())
            .map(|(idx, field)| {
                let field_ty = &field.ty;
                let meta = field_attrs.meta(&idx);
                quote! {
                    UnnamedField::new::<#field_ty>(#meta)
                }
            });

        let meta = attrs.meta();

        quote! {
            fn type_info(&self) -> TypeInfo {
                impl Typed for #ident {
                    fn type_info() -> TypeInfo {
                        let fields = &[#(#code_for_fields),*];
                        TupleStructInfo::new::<Self>(fields, #meta).into()
                    }
                }

                <Self as Typed>::type_info()
            }
        }
    };

    let fn_debug = attrs.fn_debug_tokens();
    let fn_clone_reflect = attrs.fn_clone_reflect_tokens();

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

            #fn_type_info
            #fn_patch
            #fn_to_value
            #fn_clone_reflect
            #fn_debug

            fn reflect_ref(&self) -> ReflectRef<'_> {
                ReflectRef::TupleStruct(self)
            }

            fn reflect_mut(&mut self) -> ReflectMut<'_> {
                ReflectMut::TupleStruct(self)
            }
        }
    }
}

fn expand_from_reflect(
    ident: &Ident,
    fields: &Fields,
    field_attrs: &AttrsDatabase<usize>,
) -> TokenStream {
    let fn_from_reflect = {
        let code_for_fields = fields.iter().enumerate().map(|(idx, field)| {
            let field_index = Index {
                index: idx as u32,
                span: field.span(),
            };
            let ty = &field.ty;
            let span = ty.span();
            if field_attrs.skip(&idx) {
                quote_spanned! {span=>
                    #field_index: ::std::default::Default::default(),
                }
            } else {
                quote_spanned! {span=>
                    #field_index: tuple_struct
                        .element(#field_index)?
                        .downcast_ref::<#ty>()?
                        .to_owned(),
                }
            }
        });

        quote! {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                let tuple_struct = reflect.reflect_ref().as_tuple_struct()?;
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

fn expand_tuple_struct(
    ident: &Ident,
    fields: &Fields,
    field_attrs: &AttrsDatabase<usize>,
) -> TokenStream {
    let fn_element = {
        let match_arms = fields
            .iter()
            .enumerate()
            .filter(field_attrs.filter_out_skipped_unnamed())
            .map(|(idx, field)| {
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
        let match_arms = fields
            .iter()
            .enumerate()
            .filter(field_attrs.filter_out_skipped_unnamed())
            .map(|(idx, field)| {
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
        let code_for_fields = fields
            .iter()
            .enumerate()
            .filter(field_attrs.filter_out_skipped_unnamed())
            .map(|(idx, field)| {
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
        let code_for_fields = fields
            .iter()
            .enumerate()
            .filter(field_attrs.filter_out_skipped_unnamed())
            .map(|(idx, field)| {
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
