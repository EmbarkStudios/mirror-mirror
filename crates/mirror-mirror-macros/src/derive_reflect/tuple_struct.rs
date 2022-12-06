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

use super::attrs::AttrsDatabase;
use super::attrs::ItemAttrs;
use super::Generics;

type Fields = Punctuated<Field, Token![,]>;

pub(super) fn expand(
    ident: &Ident,
    fields: FieldsUnnamed,
    attrs: ItemAttrs,
    generics: &Generics<'_>,
) -> syn::Result<TokenStream> {
    let field_attrs = AttrsDatabase::new_from_unnamed(&fields)?;

    let fields = fields.unnamed;

    let reflect = expand_reflect(ident, &fields, &attrs, &field_attrs, generics);
    let from_reflect = expand_from_reflect(ident, &attrs, &fields, &field_attrs, generics);
    let tuple_struct = expand_tuple_struct(ident, &fields, &field_attrs, generics);

    Ok(quote! {
        #reflect
        #from_reflect
        #tuple_struct
    })
}

fn expand_reflect(
    ident: &Ident,
    fields: &Fields,
    attrs: &ItemAttrs,
    field_attrs: &AttrsDatabase<usize>,
    generics: &Generics<'_>,
) -> TokenStream {
    let fn_patch = {
        let code_for_fields = fields
            .iter()
            .enumerate()
            .filter(field_attrs.filter_out_skipped_unnamed())
            .map(|(idx, _)| {
                quote! {
                    if let Some(new_value) = tuple_struct.field(#idx) {
                        self.field_mut(#idx).unwrap().patch(new_value);
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
                    let value = value.with_field(self.#field_index.to_value());
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
                let docs = field_attrs.docs(&idx);
                quote! {
                    UnnamedFieldNode::new::<#field_ty>(#meta, #docs, graph)
                }
            });

        let meta = attrs.meta();
        let docs = attrs.docs();
        let Generics {
            impl_generics,
            type_generics,
            where_clause,
        } = generics;

        quote! {
            fn type_info(&self) -> TypeInfoRoot {
                impl #impl_generics Typed for #ident #type_generics #where_clause {
                    fn build(graph: &mut TypeInfoGraph) -> Id {
                        let fields = &[#(#code_for_fields),*];
                        graph.get_or_build_with::<Self, _>(|graph| {
                            TupleStructInfoNode::new::<Self>(fields, #meta, #docs)
                        })
                    }
                }

                <Self as Typed>::type_info()
            }
        }
    };

    let fn_debug = attrs.fn_debug_tokens();
    let fn_clone_reflect = attrs.fn_clone_reflect_tokens();
    let Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = generics;

    quote! {
        impl #impl_generics Reflect for #ident #type_generics #where_clause {
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
    attrs: &ItemAttrs,
    fields: &Fields,
    field_attrs: &AttrsDatabase<usize>,
    generics: &Generics<'_>,
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
                    #field_index: ::core::default::Default::default(),
                }
            } else if attrs.clone_opt_out {
                quote_spanned! {span=>
                    #field_index: {
                        let value = tuple_struct.field(#field_index)?;
                        <#ty as FromReflect>::from_reflect(value)?
                    },
                }
            } else {
                quote_spanned! {span=>
                    #field_index: {
                        let value = tuple_struct.field(#field_index)?;
                        if let Some(value) = value.downcast_ref::<#ty>() {
                            value.to_owned()
                        } else {
                            <#ty as FromReflect>::from_reflect(value)?.to_owned()
                        }
                    },
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

    let Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = generics;

    quote! {
        impl #impl_generics FromReflect for #ident #type_generics #where_clause {
            #fn_from_reflect
        }
    }
}

fn expand_tuple_struct(
    ident: &Ident,
    fields: &Fields,
    field_attrs: &AttrsDatabase<usize>,
    generics: &Generics<'_>,
) -> TokenStream {
    let fn_field = {
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
            fn field(&self, index: usize) -> Option<&dyn Reflect> {
                match index {
                    #(#match_arms)*
                    _ => None,
                }
            }
        }
    };

    let fn_field_mut = {
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
            fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
                match index {
                    #(#match_arms)*
                    _ => None,
                }
            }
        }
    };

    let fn_fields = {
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
            fn fields(&self) -> ValueIter<'_> {
                let iter = [#(#code_for_fields)*];
                ValueIter::new(iter)
            }
        }
    };

    let fn_fields_mut = {
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
            fn fields_mut(&mut self) -> ValueIterMut<'_> {
                let iter = [#(#code_for_fields)*];
                ValueIterMut::new(iter)
            }
        }
    };

    let Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = generics;

    quote! {
        impl #impl_generics TupleStruct for #ident #type_generics #where_clause {
            #fn_field
            #fn_field_mut
            #fn_fields
            #fn_fields_mut
        }
    }
}
