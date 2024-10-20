extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

// Attribute macro for schema generation
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
            fn schema() -> String {
                #schema_impl
            }
        }
    };

    TokenStream::from(expanded)
}

// Generates schema for atom types (just the type name)
fn generate_atom_schema(name: &syn::Ident) -> proc_macro2::TokenStream {
    let type_name = format!("{}", quote!(#name));
    quote! {
        #type_name.to_string()
    }
}

// Generates structural schema (tuple-like for structs, no names for enum variants)
fn generate_structural_schema(data: &syn::Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let types: Vec<proc_macro2::TokenStream> = fields.named.iter().map(|f| {
                        let ty = &f.ty;
                        quote! {
                            <#ty as Schema>::schema()
                        }
                    }).collect();
                    quote! {
                        format!("({})", vec![#(#types),*].join(","))
                    }
                }
                Fields::Unnamed(fields) => {
                    let types: Vec<proc_macro2::TokenStream> = fields.unnamed.iter().map(|f| {
                        let ty = &f.ty;
                        quote! {
                            <#ty as Schema>::schema()
                        }
                    }).collect();
                    quote! {
                        format!("({})", vec![#(#types),*].join(","))
                    }
                }
                Fields::Unit => quote! {
                    "()".to_string()
                },
            }
        }
        Data::Enum(data_enum) => {
            let variant_schemas: Vec<proc_macro2::TokenStream> = data_enum.variants.iter().map(|v| {
                let variant_fields = match &v.fields {
                    Fields::Named(fields) => fields.named.iter().map(|f| {
                        let ty = &f.ty;
                        quote! {
                            <#ty as Schema>::schema()
                        }
                    }).collect(),
                    Fields::Unnamed(fields) => fields.unnamed.iter().map(|f| {
                        let ty = &f.ty;
                        quote! {
                            <#ty as Schema>::schema()
                        }
                    }).collect(),
                    Fields::Unit => vec![],
                };
                quote! {
                    format!("({})", vec![#(#variant_fields),*].join(","))
                }
            }).collect();
            quote! {
                vec![#(#variant_schemas),*].join("|")
            }
        }
        _ => panic!("Unsupported type for Structural schema"),
    }
}

// Generates nominal schema (with field/variant names)
fn generate_nominal_schema(name: &syn::Ident, data: &syn::Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let field_schemas: Vec<proc_macro2::TokenStream> = fields.named.iter().map(|f| {
                        let field_name = f.ident.as_ref().unwrap().to_string();
                        let field_type = &f.ty;
                        quote! {
                            format!("{}:{}", #field_name, <#field_type as Schema>::schema())
                        }
                    }).collect();
                    quote! {
                        format!("{}{{{}}}", stringify!(#name), vec![#(#field_schemas),*].join(","))
                    }
                }
                Fields::Unnamed(fields) => {
                    let field_schemas: Vec<proc_macro2::TokenStream> = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let field_type = &f.ty;
                        quote! {
                            format!("field{}:{}", i, <#field_type as Schema>::schema())
                        }
                    }).collect();
                    quote! {
                        format!("{}({})", stringify!(#name), vec![#(#field_schemas),*].join(","))
                    }
                }
                Fields::Unit => quote! {
                    format!("{}()", stringify!(#name))
                },
            }
        }
        Data::Enum(data_enum) => {
            let variants: Vec<proc_macro2::TokenStream> = data_enum.variants.iter().map(|v| {
                let variant_name = &v.ident;
                let variant_fields: Vec<proc_macro2::TokenStream> = match &v.fields {
                    Fields::Named(fields) => fields.named.iter().map(|f| {
                        let field_type = &f.ty;
                        quote! {
                            <#field_type as Schema>::schema()
                        }
                    }).collect(),
                    Fields::Unnamed(fields) => fields.unnamed.iter().map(|f| {
                        let field_type = &f.ty;
                        quote! {
                            <#field_type as Schema>::schema()
                        }
                    }).collect(),
                    Fields::Unit => vec![],
                };
                quote! {
                    format!("{}({})", stringify!(#variant_name), vec![#(#variant_fields),*].join(","))
                }
            }).collect();
            quote! {
                vec![#(#variants),*].join("|")
            }
        }
        _ => panic!("Unsupported type for Nominal schema"),
    }
}
