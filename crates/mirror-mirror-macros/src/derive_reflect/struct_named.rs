use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Field;
use syn::FieldsNamed;
use syn::Ident;
use syn::Token;

use super::attrs::AttrsDatabase;
use super::attrs::ItemAttrs;
use super::Generics;
use crate::stringify;

type Fields = Punctuated<Field, Token![,]>;

pub(super) fn expand(
    ident: &Ident,
    fields: FieldsNamed,
    attrs: ItemAttrs,
    generics: &Generics<'_>,
) -> syn::Result<TokenStream> {
    let field_attrs = AttrsDatabase::new_from_named(&fields)?;

    let fields = fields.named;

    let describe_type = expand_describe_type(ident, &fields, &attrs, &field_attrs, generics);
    let default_value = expand_default_value(ident, &attrs, generics);
    let reflect = expand_reflect(ident, &fields, &attrs, &field_attrs, generics);
    let from_reflect = (!attrs.from_reflect_opt_out)
        .then(|| expand_from_reflect(ident, &attrs, &fields, &field_attrs, generics));
    let struct_ = expand_struct(ident, &fields, &attrs, &field_attrs, generics);

    Ok(quote! {
        #describe_type
        #default_value
        #reflect
        #from_reflect
        #struct_
    })
}

fn expand_describe_type(
    ident: &Ident,
    fields: &Fields,
    attrs: &ItemAttrs,
    field_attrs: &AttrsDatabase<Ident>,
    generics: &Generics<'_>,
) -> TokenStream {
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

    let meta = attrs.meta();
    let docs = attrs.docs();
    let Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = generics;

    quote! {
        impl #impl_generics DescribeType for #ident #type_generics #where_clause {
            fn build(graph: &mut TypeGraph) -> NodeId {
                graph.get_or_build_node_with::<Self, _>(|graph| {
                    let fields = &[#(#code_for_fields),*];
                    StructNode::new::<Self>(fields, #meta, #docs)
                })
            }
        }
    }
}

fn expand_default_value(ident: &Ident, attrs: &ItemAttrs, generics: &Generics<'_>) -> TokenStream {
    let Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = generics;

    let fn_default_value = if attrs.default_opt_out {
        quote! {
            fn default_value() -> Option<Value> {
                None
            }
        }
    } else {
        quote! {
            fn default_value() -> Option<Value> {
                fn __default<Z: Default>() -> Z {
                    Default::default()
                }
                Some(__default::<#ident #type_generics>().to_value())
            }
        }
    };

    quote! {
        impl #impl_generics DefaultValue for #ident #type_generics #where_clause {
            #fn_default_value
        }
    }
}

fn expand_reflect(
    ident: &Ident,
    fields: &Fields,
    attrs: &ItemAttrs,
    field_attrs: &AttrsDatabase<Ident>,
    generics: &Generics<'_>,
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
                    let value = value.with_field(#field, self.#ident.to_value());
                }
            });

        let fields_len = fields.len();

        quote! {
            fn to_value(&self) -> Value {
                let value = StructValue::with_capacity(#fields_len);
                #(#code_for_fields)*
                value.into()
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

            #fn_patch
            #fn_to_value
            #fn_clone_reflect
            #fn_debug

            fn reflect_owned(self: Box<Self>) -> ReflectOwned {
                ReflectOwned::Struct(self)
            }

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
    attrs: &ItemAttrs,
    fields: &Fields,
    field_attrs: &AttrsDatabase<Ident>,
    generics: &Generics<'_>,
) -> TokenStream {
    let fn_from_reflect = {
        let code_for_fields = fields.iter().map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let skip = field_attrs.skip(ident);
            let span = field.ty.span();

            if skip {
                quote_spanned! {span=>
                    #ident: ::core::default::Default::default(),
                }
            } else {
                let ty = &field.ty;
                let field = stringify(ident);
                if let Some(from_reflect_with) = field_attrs.from_reflect_with(ident) {
                    quote_spanned! {span=>
                        #ident: {
                            let value = struct_.field(#field)?;
                            #from_reflect_with(value)?
                        },
                    }
                } else if attrs.clone_opt_out {
                    quote_spanned! {span=>
                        #ident: {
                            let value = struct_.field(#field)?;
                            <#ty as FromReflect>::from_reflect(value)?
                        },
                    }
                } else {
                    quote_spanned! {span=>
                        #ident: {
                            let value = struct_.field(#field)?;
                            if let Some(value) = value.downcast_ref::<#ty>() {
                                value.clone()
                            } else {
                                <#ty as FromReflect>::from_reflect(value)?
                            }
                        },
                    }
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

fn expand_struct(
    ident: &Ident,
    fields: &Fields,
    attrs: &ItemAttrs,
    field_attrs: &AttrsDatabase<Ident>,
    generics: &Generics<'_>,
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

    let fn_field_at = {
        let code_for_fields = fields
            .iter()
            .filter(field_attrs.filter_out_skipped_named())
            .enumerate()
            .map(|(index, field)| {
                let ident = &field.ident;
                quote! {
                    if index == #index {
                        return Some(&self.#ident);
                    }
                }
            });

        quote! {
            fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
                #(#code_for_fields)*
                None
            }
        }
    };

    let fn_field_at_mut = {
        let code_for_fields = fields
            .iter()
            .filter(field_attrs.filter_out_skipped_named())
            .enumerate()
            .map(|(index, field)| {
                let ident = &field.ident;
                quote! {
                    if index == #index {
                        return Some(&mut self.#ident);
                    }
                }
            });

        quote! {
            fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
                #(#code_for_fields)*
                None
            }
        }
    };

    let fn_name_at = {
        let code_for_fields = fields
            .iter()
            .filter(field_attrs.filter_out_skipped_named())
            .enumerate()
            .map(|(index, field)| {
                let ident = &field.ident;
                quote! {
                    if index == #index {
                        return Some(::core::stringify!(#ident));
                    }
                }
            });

        quote! {
            fn name_at(&self, index: usize) -> Option<&str> {
                #(#code_for_fields)*
                None
            }
        }
    };

    let fields_len = fields
        .iter()
        .filter(field_attrs.filter_out_skipped_named())
        .count();

    let fn_fields_len = {
        quote! {
            fn fields_len(&self) -> usize {
                #fields_len
            }
        }
    };

    let fn_fields = {
        let crate_name = &attrs.crate_name;
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

        if fields_len <= 32 {
            quote! {
                fn fields(&self) -> #crate_name::struct_::FieldsIter<'_> {
                    let fields = [
                        #(#code_for_fields)*
                    ];
                    Box::new(fields.into_iter())
                }
            }
        } else {
            quote! {
                fn fields(&self) -> #crate_name::struct_::FieldsIter<'_> {
                    let fields = vec![
                        #(#code_for_fields)*
                    ];
                    Box::new(fields.into_iter())
                }
            }
        }
    };

    let fn_fields_mut = {
        let crate_name = &attrs.crate_name;
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

        if fields_len <= 32 {
            quote! {
                fn fields_mut(&mut self) -> #crate_name::struct_::FieldsIterMut <'_> {
                    let fields = [
                        #(#code_for_fields)*
                    ];
                    Box::new(fields.into_iter())
                }
            }
        } else {
            quote! {
                fn fields_mut(&mut self) -> #crate_name::struct_::FieldsIterMut <'_> {
                    let fields = vec![
                        #(#code_for_fields)*
                    ];
                    Box::new(fields.into_iter())
                }
            }
        }
    };

    let Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = generics;

    quote! {
        impl #impl_generics Struct for #ident #type_generics #where_clause {
            #fn_field
            #fn_field_mut
            #fn_field_at
            #fn_field_at_mut
            #fn_name_at
            #fn_fields
            #fn_fields_mut
            #fn_fields_len
        }
    }
}
