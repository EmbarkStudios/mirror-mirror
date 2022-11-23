use crate::stringify;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, Ident, Variant};

pub(crate) fn expand(ident: &Ident, enum_: DataEnum) -> syn::Result<TokenStream> {
    let reflect = expand_reflect(ident, &enum_);
    let from_reflect = expand_from_reflect(ident, &enum_);
    let enum_ = expand_enum(ident, &enum_);

    Ok(quote! {
        #reflect
        #from_reflect
        #enum_
    })
}

fn expand_reflect(ident: &Ident, enum_: &DataEnum) -> TokenStream {
    let fn_patch = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let (ident, field_names) = variant_parts(variant);

            let set_fields = variant.fields.iter().map(|field| {
                let ident = field.ident.as_ref().expect("unnamed field in variant");
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

            let set_fields = variant.fields.iter().map(|field| {
                let ident = field.ident.as_ref().expect("unnamed field in variant");
                let ident_string = stringify(ident);
                quote! {
                    value.set_field(#ident_string, #ident.to_owned());
                }
            });

            quote! {
                Self::#ident { #(#field_names,)* } => {
                    let mut value = EnumValue::new(#ident_string);
                    #(#set_fields)*
                    value.into()
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

fn expand_from_reflect(ident: &Ident, enum_: &DataEnum) -> TokenStream {
    let match_arms = enum_.variants.iter().map(|variant| {
        let ident = &variant.ident;
        let ident_string = stringify(&variant.ident);

        let set_fields = variant.fields.iter().map(|field| {
            let ident = field.ident.as_ref().expect("unnamed field in variant");
            let ident_string = stringify(ident);
            let ty = &field.ty;
            quote! {
                #ident: enum_.get_field::<#ty>(#ident_string)?.to_owned(),
            }
        });

        quote! {
            #ident_string => Some(Self::#ident {
                #(#set_fields)*
            }),
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

    let fn_field = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let (ident, field_names) = variant_parts(variant);

            let return_if_name_matches = variant.fields.iter().map(|field| {
                let ident = field.ident.as_ref().expect("unnamed field in variant");
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
        });

        quote! {
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

            let return_if_name_matches = variant.fields.iter().map(|field| {
                let ident = field.ident.as_ref().expect("unnamed field in variant");
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
        });

        quote! {
            fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
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

            let code_for_fields = variant.fields.iter().map(|field| {
                let ident = &field.ident;
                let field = stringify(ident);
                quote! {
                    (#field, #ident.as_reflect()),
                }
            });

            quote! {
                Self::#ident { #(#field_names,)* } => {
                    let iter = [#(#code_for_fields)*];
                    PairIter::new(iter)
                }
            }
        });

        quote! {
            fn fields(&self) -> PairIter<'_> {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    let fn_fields_mut = {
        let match_arms = enum_.variants.iter().map(|variant| {
            let (ident, field_names) = variant_parts(variant);

            let code_for_fields = variant.fields.iter().map(|field| {
                let ident = &field.ident;
                let field = stringify(ident);
                quote! {
                    (#field, #ident.as_reflect_mut()),
                }
            });

            quote! {
                Self::#ident { #(#field_names,)* } => {
                    let iter = [#(#code_for_fields)*];
                    PairIterMut::new(iter)
                }
            }
        });

        quote! {
            fn fields_mut(&mut self) -> PairIterMut<'_> {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    quote! {
        impl Enum for #ident {
            #fn_variant_name
            #fn_field
            #fn_field_mut
            #fn_fields
            #fn_fields_mut
        }
    }
}

fn variant_parts(variant: &Variant) -> (&Ident, impl Iterator<Item = &Ident>) {
    let ident = &variant.ident;

    let field_names = variant
        .fields
        .iter()
        .map(|field| field.ident.as_ref().expect("unnamed field in variant"));

    (ident, field_names)
}
