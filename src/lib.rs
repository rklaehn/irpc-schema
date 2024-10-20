extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// The attribute macro for schema generation
#[proc_macro_attribute]
pub fn schema(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as syn::Ident);
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let schema_impl = match attr.to_string().as_str() {
        "Atom" => generate_atom_schema(&name),
        "Structural" => generate_structural_schema(&input.data),
        "Nominal" => generate_nominal_schema(&name, &input.data),
        _ => panic!("Unsupported schema type"),
    };

    let expanded = quote! {
        #input

        impl Schema for #name {
            fn schema() -> ReifiedSchema {
                #schema_impl
            }
        }
    };
    println!("{}", expanded);

    TokenStream::from(expanded)
}

// Generates an Atom schema (just the type name)
fn generate_atom_schema(name: &syn::Ident) -> proc_macro2::TokenStream {
    let type_name = format!("{}", quote!(#name));
    quote! {
        ReifiedSchema::Atom(#type_name.to_string())
    }
}

// Generates a Structural schema (tuples or unnamed structs)
fn generate_structural_schema(data: &syn::Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let types: Vec<proc_macro2::TokenStream> = fields
                    .named
                    .iter()
                    .map(|f| {
                        let ty = &f.ty;
                        quote! {
                            <#ty as Schema>::schema()
                        }
                    })
                    .collect();
                if types.is_empty() {
                    quote! {
                        ReifiedSchema::Unit
                    }
                } else {
                    quote! {
                        ReifiedSchema::Product(vec![#(#types),*])
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let types: Vec<proc_macro2::TokenStream> = fields
                    .unnamed
                    .iter()
                    .map(|f| {
                        let ty = &f.ty;
                        quote! {
                            <#ty as Schema>::schema()
                        }
                    })
                    .collect();
                if types.is_empty() {
                    quote! {
                        ReifiedSchema::Unit
                    }
                } else {
                    quote! {
                        ReifiedSchema::Product(vec![#(#types),*])
                    }
                }
            }
            Fields::Unit => quote! {
                ReifiedSchema::Unit
            },
        },
        Data::Enum(data_enum) => {
            let variant_schemas: Vec<proc_macro2::TokenStream> = data_enum
                .variants
                .iter()
                .map(|v| {
                    let variant_fields = match &v.fields {
                        Fields::Named(fields) => fields
                            .named
                            .iter()
                            .map(|f| {
                                let ty = &f.ty;
                                quote! {
                                    <#ty as Schema>::schema()
                                }
                            })
                            .collect(),
                        Fields::Unnamed(fields) => fields
                            .unnamed
                            .iter()
                            .map(|f| {
                                let ty = &f.ty;
                                quote! {
                                    <#ty as Schema>::schema()
                                }
                            })
                            .collect(),
                        Fields::Unit => vec![],
                    };
                    if variant_fields.is_empty() {
                        quote! {
                            ReifiedSchema::Unit
                        }
                    } else {
                        quote! {
                            ReifiedSchema::Product(vec![#(#variant_fields),*])
                        }
                    }
                })
                .collect();
            if variant_schemas.is_empty() {
                return quote! {
                    ReifiedSchema::Bottom
                };
            }
            quote! {
                ReifiedSchema::Sum(vec![#(#variant_schemas),*])
            }
        }
        _ => panic!("Unsupported type for Structural schema"),
    }
}

// Generates a Nominal schema (Struct or Enum with names)
fn generate_nominal_schema(name: &syn::Ident, data: &syn::Data) -> proc_macro2::TokenStream {
    let name_text = name.to_string();
    match data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let field_schemas: Vec<proc_macro2::TokenStream> = fields
                    .named
                    .iter()
                    .map(|f| {
                        let field_name = f.ident.as_ref().unwrap().to_string();
                        let field_type = &f.ty;
                        quote! {
                            Named(#field_name.to_string(), <#field_type as Schema>::schema())
                        }
                    })
                    .collect();
                let schema = if field_schemas.is_empty() {
                    quote! { ReifiedSchema::Unit }
                } else {
                    quote! { ReifiedSchema::Struct(vec![#(#field_schemas),*]) }
                };
                quote! {
                    ReifiedSchema::Named(
                        Box::new(Named(#name_text.to_string(), #schema))
                    )
                }
            }
            Fields::Unnamed(fields) => {
                let field_schemas: Vec<proc_macro2::TokenStream> = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(_i, f)| {
                        let field_type = &f.ty;
                        quote! {
                            <#field_type as Schema>::schema()
                        }
                    })
                    .collect();
                let schema = if field_schemas.is_empty() {
                    quote! { ReifiedSchema::Unit }
                } else {
                    quote! { ReifiedSchema::Product(vec![#(#field_schemas),*]) }
                };
                quote! {
                    ReifiedSchema::Named(
                        Box::new(Named(#name_text.to_string(), #schema))
                    )
                }
            }
            Fields::Unit => quote! {
                ReifiedSchema::Named(
                    Box::new(Named(#name_text.to_string(), ReifiedSchema::Unit))
                )
            },
        },
        Data::Enum(data_enum) => {
            let variants: Vec<proc_macro2::TokenStream> = data_enum
                .variants
                .iter()
                .map(|v| {
                    let variant_name = &v.ident;
                    let variant_name_text = variant_name.to_string();
                    match &v.fields {
                        Fields::Named(fields) => {
                            let named = fields
                                .named
                                .iter()
                                .map(|f| {
                                    let field_type = &f.ty;
                                    let field_name = f.ident.as_ref().unwrap().to_string();
                                    quote! {
                                        Named(#field_name.to_string(),<#field_type as Schema>::schema())
                                    }
                                })
                                .collect::<Vec<_>>();
                            let schema_type = if named.is_empty() {
                                quote! { ReifiedSchema::Unit }
                            } else if named.len() == 1 {
                                quote! { ReifiedSchema::Struct(vec![#(#named),*]) }
                            } else {
                                quote! { ReifiedSchema::Enum(vec![#(#named),*]) }
                            };
                            quote! {
                                Named(
                                    #variant_name_text.to_string(),
                                    #schema_type
                                )
                            }
                        }
                        Fields::Unnamed(fields) => {
                            let unnamed = fields
                                .unnamed
                                .iter()
                                .map(|f| {
                                    let field_type = &f.ty;
                                    quote! {
                                        <#field_type as Schema>::schema()
                                    }
                                })
                                .collect::<Vec<_>>();
                            let schema_type = if unnamed.is_empty() {
                                quote! { ReifiedSchema::Unit }
                            } else if unnamed.len() == 1 {
                                quote! { ReifiedSchema::Product(vec![#(#unnamed),*]) }
                            } else {
                                quote! { ReifiedSchema::Sum(vec![#(#unnamed),*]) }
                            };
                            quote! {
                                Named(
                                    #variant_name_text.to_string(),
                                    #schema_type
                                )
                            }
                        }
                        Fields::Unit => {
                            quote! {
                                Named(
                                    #variant_name_text.to_string(),
                                    ReifiedSchema::Unit
                                )
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();

            let name_text = name.to_string();
            let schema = if variants.is_empty() {
                quote! { ReifiedSchema::Bottom }
            } else if variants.len() == 1 {
                quote! { ReifiedSchema::Struct(vec![#(#variants),*]) }
            } else {
                quote! { ReifiedSchema::Enum(vec![#(#variants),*]) }
            };
            quote! {
                ReifiedSchema::Named(
                    Box::new(Named(#name_text.to_string(), #schema))
                )
            }
        }
        _ => panic!("Unsupported type for Nominal schema"),
    }
}
