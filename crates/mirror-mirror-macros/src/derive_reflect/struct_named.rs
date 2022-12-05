use super::attrs::AttrsDatabase;
use super::attrs::ItemAttrs;
use crate::stringify;
use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Field;
use syn::FieldsNamed;
use syn::Ident;
use syn::Token;

type Fields = Punctuated<Field, Token![,]>;

pub(super) fn expand(
    ident: &Ident,
    fields: FieldsNamed,
    item_attrs: ItemAttrs,
) -> syn::Result<TokenStream> {
    let field_attrs = AttrsDatabase::new_from_named(&fields)?;

    let fields = fields.named;

    let reflect = expand_reflect(ident, &fields, item_attrs, &field_attrs);
    let from_reflect = expand_from_reflect(ident, &fields, &field_attrs);
    let struct_ = expand_struct(ident, &fields, &field_attrs);

    Ok(quote! {
        #reflect
        #from_reflect
        #struct_
    })
}

fn expand_reflect(
    ident: &Ident,
    fields: &Fields,
    item_attrs: ItemAttrs,
    field_attrs: &AttrsDatabase<Ident>,
) -> TokenStream {
    let fn_patch = {
        let code_for_fields = fields
            .iter()
            .filter(field_attrs.filter_out_skipped_named())
            .map(|field| {
                let field = stringify(&field.ident);
                quote! {
                    if let Some(field) = value.field(#field) {
                        self.field_mut(#field).unwrap().patch(field);
                    }
                }
            });

        quote! {
            fn patch(&mut self, value: &dyn Reflect) {
                if let Some(value) = value.reflect_ref().as_struct() {
                    #(#code_for_fields)*
                }
            }
        }
    };

    let fn_to_value = {
        let code_for_fields = fields
            .iter()
            .filter(field_attrs.filter_out_skipped_named())
            .map(|field| {
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

    let fn_type_info = {
        let code_for_fields = fields
            .iter()
            .filter(field_attrs.filter_out_skipped_named())
            .map(|field| {
                let name = stringify(&field.ident);
                let field_ty = &field.ty;
                let ident = field.ident.as_ref().unwrap();
                let meta = field_attrs.meta(ident);
                let docs = field_attrs.docs(ident);
                quote! {
                    NamedFieldNode::new::<#field_ty>(#name, #meta, #docs, graph)
                }
            });

        let meta = item_attrs.meta();
        let docs = item_attrs.docs();

        quote! {
            fn type_info(&self) -> TypeInfoRoot {
                impl Typed for #ident {
                    fn build(graph: &mut TypeInfoGraph) -> Id {
                        graph.get_or_build_with::<#ident, _>(|graph| {
                            let fields = &[#(#code_for_fields),*];
                            StructInfoNode::new::<#ident>(fields, #meta, #docs)
                        })
                    }
                }

                <Self as Typed>::type_info()
            }
        }
    };

    let fn_debug = item_attrs.fn_debug_tokens();
    let fn_clone_reflect = item_attrs.fn_clone_reflect_tokens();

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
                ReflectRef::Struct(self)
            }

            fn reflect_mut(&mut self) -> ReflectMut<'_> {
                ReflectMut::Struct(self)
            }
        }
    }
}

fn expand_from_reflect(
    ident: &Ident,
    fields: &Fields,
    field_attrs: &AttrsDatabase<Ident>,
) -> TokenStream {
    let fn_from_reflect = {
        let code_for_fields = fields.iter().map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let skip = field_attrs.skip(ident);
            let span = field.ty.span();

            if skip {
                quote_spanned! {span=>
                    #ident: ::std::default::Default::default(),
                }
            } else {
                let ty = &field.ty;
                let field = stringify(ident);
                quote_spanned! {span=>
                    #ident: <#ty as FromReflect>::from_reflect(struct_.field(#field)?)?.to_owned(),
                }
            }
        });

        quote! {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                let struct_ = reflect.reflect_ref().as_struct()?;
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

fn expand_struct(
    ident: &Ident,
    fields: &Fields,
    field_attrs: &AttrsDatabase<Ident>,
) -> TokenStream {
    let fn_field = {
        let code_for_fields = fields
            .iter()
            .filter(field_attrs.filter_out_skipped_named())
            .map(|field| {
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
        let code_for_fields = fields
            .iter()
            .filter(field_attrs.filter_out_skipped_named())
            .map(|field| {
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
        let code_for_fields = fields
            .iter()
            .filter(field_attrs.filter_out_skipped_named())
            .map(|field| {
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
        let code_for_fields = fields
            .iter()
            .filter(field_attrs.filter_out_skipped_named())
            .map(|field| {
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
