use crate::stringify;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, Ident, Variant};

use super::ItemAttrs;

pub(super) fn expand(ident: &Ident, enum_: DataEnum, attrs: ItemAttrs) -> syn::Result<TokenStream> {
    let reflect = expand_reflect(ident, &enum_, attrs);
    let from_reflect = expand_from_reflect(ident, &enum_);
    let enum_ = expand_enum(ident, &enum_);

    Ok(quote! {
        #reflect
        #from_reflect
        #enum_
    })
}

fn expand_reflect(ident: &Ident, enum_: &DataEnum, attrs: ItemAttrs) -> TokenStream {
    let fn_patch = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let (ident, field_names) = variant_parts(variant);

            match &variant.fields {
                syn::Fields::Named(_) => {
                    let set_fields = variant.fields.iter().map(|field| {
                        let ident = field
                            .ident
                            .as_ref()
                            .expect("named variant with unnamed field");
                        let ident_string = stringify(ident);
                        quote! {
                            if let Some(new_value) = enum_.field(#ident_string) {
                                #ident.patch(new_value);
                            }
                        }
                    });

                    quote! {
                        Self::#ident { #(#field_names,)* } => {
                            if variant_matches {
                                #(#set_fields)*
                            }
                        }
                    }
                }
                syn::Fields::Unnamed(_) => {
                    let field_names = field_names.collect::<Vec<_>>();

                    let set_fields = field_names.iter().enumerate().map(|(index, ident)| {
                        quote! {
                            if let Some(new_value) = enum_.element(#index) {
                                #ident.patch(new_value);
                            }
                        }
                    });

                    quote! {
                        Self::#ident(#(#field_names,)*) => {
                            if variant_matches {
                                #(#set_fields)*
                            }
                        }
                    }
                }
                syn::Fields::Unit => {
                    quote! {
                        Self::#ident => {
                            // unit variants don't have any fields to patch
                        }
                    }
                }
            }
        });

        quote! {
            fn patch(&mut self, value: &dyn Reflect) {
                if let Some(new) = value.downcast_ref::<Self>() {
                    *self = new.clone();
                } else if let Some(enum_) = value.as_enum() {
                    if let Some(new) = Self::from_reflect(value) {
                        *self = new;
                    } else {
                        let variant_matches = self.variant_name() == enum_.variant_name();
                        match self {
                            #(#match_arms)*
                        }
                    }
                }
            }
        }
    };

    let fn_to_value = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let (ident, field_names) = variant_parts(variant);
            let ident_string = stringify(ident);

            match &variant.fields {
                syn::Fields::Named(_) => {
                    let set_fields = variant.fields.iter().map(|field| {
                        let ident = field
                            .ident
                            .as_ref()
                            .expect("named variant with unnamed field");
                        let ident_string = stringify(ident);
                        quote! {
                            value.set_field(#ident_string, #ident.to_owned());
                        }
                    });

                    quote! {
                        Self::#ident { #(#field_names,)* } => {
                            let mut value = EnumValue::new_struct_variant(#ident_string);
                            #(#set_fields)*
                            value.into()
                        }
                    }
                }
                syn::Fields::Unnamed(_) => {
                    let field_names = field_names.collect::<Vec<_>>();

                    quote! {
                        Self::#ident(#(#field_names,)*) => {
                            let mut value = EnumValue::new_tuple_variant(#ident_string);
                            #(
                                value.push_element(#field_names.to_owned());
                            )*
                            value.into()
                        }
                    }
                }
                syn::Fields::Unit => {
                    quote! {
                        Self::#ident => {
                            EnumValue::new_unit_variant(#ident_string).into()
                        }
                    }
                }
            }
        });

        quote! {
            fn to_value(&self) -> Value {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    let fn_debug = attrs.fn_debug_tokens();

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
                None
            }

            fn as_tuple_struct_mut(&mut self) -> Option<&mut dyn TupleStruct> {
                None
            }

            fn as_enum(&self) -> Option<&dyn Enum> {
                Some(self)
            }

            fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
                Some(self)
            }

            #fn_debug
        }
    }
}

fn expand_from_reflect(ident: &Ident, enum_: &DataEnum) -> TokenStream {
    let match_arms = enum_.variants.iter().map(|variant| {
        let (ident, _) = variant_parts(variant);
        let ident_string = stringify(ident);

        let expr = match &variant.fields {
            syn::Fields::Named(fields) => {
                let set_fields = fields.named.iter().map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    let ident_string = stringify(ident);
                    let ty = &field.ty;
                    quote! {
                        #ident: enum_.get_field::<#ty>(#ident_string)?.to_owned(),
                    }
                });

                quote! {
                    Some(Self::#ident {
                        #(#set_fields)*
                    }),
                }
            }
            syn::Fields::Unnamed(fields) => {
                let set_fields = fields.unnamed.iter().enumerate().map(|(idx, field)| {
                    let ty = &field.ty;
                    quote! {
                        enum_.get_field::<#ty>(#idx)?.to_owned(),
                    }
                });

                quote! {
                    Some(Self::#ident(#(#set_fields)*)),
                }
            }
            syn::Fields::Unit => {
                quote! {
                    Some(Self::#ident),
                }
            }
        };

        quote! {
            #ident_string => #expr
        }
    });

    quote! {
        impl FromReflect for #ident {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                let enum_ = reflect.as_enum()?;
                match enum_.variant_name() {
                    #(#match_arms)*
                    _ => None,
                }
            }
        }
    }
}

fn expand_enum(ident: &Ident, enum_: &DataEnum) -> TokenStream {
    let fn_variant_name = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let ident_string = stringify(ident);
            quote! {
                Self::#ident { .. } => #ident_string,
            }
        });

        quote! {
            fn variant_name(&self) -> &str {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    let fn_variant_kind = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let ident = &variant.ident;

            let kind = match &variant.fields {
                syn::Fields::Named(_) => quote! { VariantKind::Struct },
                syn::Fields::Unnamed(_) => quote! { VariantKind::Tuple },
                syn::Fields::Unit => quote! { VariantKind::Unit },
            };

            quote! {
                Self::#ident { .. } => #kind,
            }
        });

        quote! {
            fn variant_kind(&self) -> VariantKind {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    let fn_field = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let (ident, field_names) = variant_parts(variant);
            let field_names = field_names.collect::<Vec<_>>();

            match &variant.fields {
                syn::Fields::Named(_) => {
                    let return_if_name_matches = field_names.iter().map(|ident| {
                        let ident_string = stringify(ident);
                        quote! {
                            if name == #ident_string {
                                return Some(#ident);
                            }
                        }
                    });

                    quote! {
                        Self::#ident { #(#field_names,)* } => {
                            #(#return_if_name_matches)*
                        }
                    }
                }
                syn::Fields::Unnamed(_) => quote! {
                    Self::#ident(..) => return None,
                },
                syn::Fields::Unit => quote! {
                    Self::#ident => return None,
                },
            }
        });

        quote! {
            #[allow(unused_variables, unreachable_code)]
            fn field(&self, name: &str) -> Option<&dyn Reflect> {
                match self {
                    #(#match_arms)*
                }

                None
            }

        }
    };

    let fn_field_mut = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let (ident, field_names) = variant_parts(variant);

            match &variant.fields {
                syn::Fields::Named(_) => {
                    let field_names = field_names.collect::<Vec<_>>();

                    let return_if_name_matches = field_names.iter().map(|ident| {
                        let ident_string = stringify(ident);
                        quote! {
                            if name == #ident_string {
                                return Some(#ident);
                            }
                        }
                    });

                    quote! {
                        Self::#ident { #(#field_names,)* } => {
                            #(#return_if_name_matches)*
                        },
                    }
                }
                syn::Fields::Unnamed(_) => quote! {
                    Self::#ident(..) => return None,
                },
                syn::Fields::Unit => quote! {
                    Self::#ident => return None,
                },
            }
        });

        quote! {
            #[allow(unused_variables, unreachable_code)]
            fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
                match self {
                    #(#match_arms)*
                }

                None
            }

        }
    };

    let fn_element = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let (ident, field_names) = variant_parts(variant);

            match &variant.fields {
                syn::Fields::Named(_) => {
                    quote! {
                        Self::#ident { .. } => return None,
                    }
                }
                syn::Fields::Unnamed(_) => {
                    let field_names = field_names.collect::<Vec<_>>();

                    let return_if_index_matches =
                        field_names.iter().enumerate().map(|(idx, field_name)| {
                            quote! {
                                if #idx == index {
                                    return Some(#field_name.as_reflect());
                                }
                            }
                        });

                    quote! {
                        Self::#ident(#(#field_names,)*) => {
                            #(#return_if_index_matches)*
                        },
                    }
                }
                syn::Fields::Unit => quote! {
                    Self::#ident => return None,
                },
            }
        });

        quote! {
            #[allow(unused_variables, unreachable_code)]
            fn element(&self, index: usize) -> Option<&dyn Reflect> {
                match self {
                    #(#match_arms)*
                }

                None
            }
        }
    };

    let fn_element_mut = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let (ident, field_names) = variant_parts(variant);

            match &variant.fields {
                syn::Fields::Named(_) => {
                    quote! {
                        Self::#ident { .. } => return None,
                    }
                }
                syn::Fields::Unnamed(_) => {
                    let field_names = field_names.collect::<Vec<_>>();

                    let return_if_index_matches =
                        field_names.iter().enumerate().map(|(idx, field_name)| {
                            quote! {
                                if #idx == index {
                                    return Some(#field_name.as_reflect_mut());
                                }
                            }
                        });

                    quote! {
                        Self::#ident(#(#field_names,)*) => {
                            #(#return_if_index_matches)*
                        },
                    }
                }
                syn::Fields::Unit => quote! {
                    Self::#ident => return None,
                },
            }
        });

        quote! {
            #[allow(unused_variables, unreachable_code)]
            fn element_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
                match self {
                    #(#match_arms)*
                }

                None
            }
        }
    };

    let fn_fields = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let (ident, field_names) = variant_parts(variant);
            let field_names = field_names.collect::<Vec<_>>();

            match &variant.fields {
                syn::Fields::Named(fields_named) => {
                    let code_for_fields = fields_named.named.iter().map(|field| {
                        let ident = &field.ident;
                        let field = stringify(ident);
                        quote! {
                            (#field, #ident.as_reflect()),
                        }
                    });

                    quote! {
                        Self::#ident { #(#field_names,)* } => {
                            let iter = [#(#code_for_fields)*];
                            VariantFieldIter::new_struct_variant(iter)
                        },
                    }
                }
                syn::Fields::Unnamed(_) => {
                    quote! {
                        Self::#ident(#(#field_names,)*) => {
                            let iter = [#(#field_names.as_reflect(),)*];
                            VariantFieldIter::new_tuple_variant(iter)
                        },
                    }
                }
                syn::Fields::Unit => quote! {
                    Self::#ident => {
                        VariantFieldIter::empty()
                    },
                },
            }
        });

        quote! {
            fn fields(&self) -> VariantFieldIter<'_> {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    let fn_fields_mut = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let (ident, field_names) = variant_parts(variant);
            let field_names = field_names.collect::<Vec<_>>();

            match &variant.fields {
                syn::Fields::Named(fields_named) => {
                    let code_for_fields = fields_named.named.iter().map(|field| {
                        let ident = &field.ident;
                        let field = stringify(ident);
                        quote! {
                            (#field, #ident.as_reflect_mut()),
                        }
                    });

                    quote! {
                        Self::#ident { #(#field_names,)* } => {
                            let iter = [#(#code_for_fields)*];
                            VariantFieldIterMut::new_struct_variant(iter)
                        },
                    }
                }
                syn::Fields::Unnamed(_) => {
                    quote! {
                        Self::#ident(#(#field_names,)*) => {
                            let iter = [#(#field_names.as_reflect_mut(),)*];
                            VariantFieldIterMut::new_tuple_variant(iter)
                        },
                    }
                }
                syn::Fields::Unit => quote! {
                    Self::#ident => {
                        VariantFieldIterMut::empty()
                    },
                },
            }
        });

        quote! {
            fn fields_mut(&mut self) -> VariantFieldIterMut<'_> {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    quote! {
        impl Enum for #ident {
            #fn_variant_name
            #fn_variant_kind
            #fn_field
            #fn_field_mut
            #fn_element
            #fn_element_mut
            #fn_fields
            #fn_fields_mut
        }
    }
}

fn variant_parts(variant: &Variant) -> (&Ident, impl Iterator<Item = Ident> + '_) {
    let ident = &variant.ident;

    let field_names = variant.fields.iter().enumerate().map(|(index, field)| {
        if let Some(ident) = &field.ident {
            ident.clone()
        } else {
            quote::format_ident!("field_{index}")
        }
    });

    (ident, field_names)
}
