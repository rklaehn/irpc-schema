extern crate proc_macro;
use std::collections::{HashMap, HashSet};

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Type, Ident, Error, spanned::Spanned};
use syn::{ItemEnum};

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

        impl HasSchema for #name {
            fn schema() -> Schema {
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
        Schema::Atom(#type_name.to_string())
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
                            <#ty as HasSchema>::schema()
                        }
                    })
                    .collect();
                if types.is_empty() {
                    quote! {
                        Schema::Unit
                    }
                } else {
                    quote! {
                        Schema::Product(vec![#(#types),*])
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
                            <#ty as HasSchema>::schema()
                        }
                    })
                    .collect();
                if types.is_empty() {
                    quote! {
                        Schema::Unit
                    }
                } else {
                    quote! {
                        Schema::Product(vec![#(#types),*])
                    }
                }
            }
            Fields::Unit => quote! {
                Schema::Unit
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
                                    <#ty as HasSchema>::schema()
                                }
                            })
                            .collect(),
                        Fields::Unnamed(fields) => fields
                            .unnamed
                            .iter()
                            .map(|f| {
                                let ty = &f.ty;
                                quote! {
                                    <#ty as HasSchema>::schema()
                                }
                            })
                            .collect(),
                        Fields::Unit => vec![],
                    };
                    if variant_fields.is_empty() {
                        quote! {
                            Schema::Unit
                        }
                    } else {
                        quote! {
                            Schema::Product(vec![#(#variant_fields),*])
                        }
                    }
                })
                .collect();
            if variant_schemas.is_empty() {
                return quote! {
                    Schema::Bottom
                };
            }
            quote! {
                Schema::Sum(vec![#(#variant_schemas),*])
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
                            Named(#field_name.to_string(), <#field_type as HasSchema>::schema())
                        }
                    })
                    .collect();
                let schema = if field_schemas.is_empty() {
                    quote! { Schema::Unit }
                } else {
                    quote! { Schema::Struct(vec![#(#field_schemas),*]) }
                };
                quote! {
                    Schema::Named(
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
                            <#field_type as HasSchema>::schema()
                        }
                    })
                    .collect();
                let schema = if field_schemas.is_empty() {
                    quote! { Schema::Unit }
                } else {
                    quote! { Schema::Product(vec![#(#field_schemas),*]) }
                };
                quote! {
                    Schema::Named(
                        Box::new(Named(#name_text.to_string(), #schema))
                    )
                }
            }
            Fields::Unit => quote! {
                Schema::Named(
                    Box::new(Named(#name_text.to_string(), Schema::Unit))
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
                                        Named(#field_name.to_string(),<#field_type as HasSchema>::schema())
                                    }
                                })
                                .collect::<Vec<_>>();
                            let schema_type = if named.is_empty() {
                                quote! { Schema::Unit }
                            } else if named.len() == 1 {
                                quote! { Schema::Struct(vec![#(#named),*]) }
                            } else {
                                quote! { Schema::Enum(vec![#(#named),*]) }
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
                                        <#field_type as HasSchema>::schema()
                                    }
                                })
                                .collect::<Vec<_>>();
                            let schema_type = if unnamed.is_empty() {
                                quote! { Schema::Unit }
                            } else if unnamed.len() == 1 {
                                quote! { Schema::Product(vec![#(#unnamed),*]) }
                            } else {
                                quote! { Schema::Sum(vec![#(#unnamed),*]) }
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
                                    Schema::Unit
                                )
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();

            let name_text = name.to_string();
            let schema = if variants.is_empty() {
                quote! { Schema::Bottom }
            } else if variants.len() == 1 {
                quote! { Schema::Struct(vec![#(#variants),*]) }
            } else {
                quote! { Schema::Enum(vec![#(#variants),*]) }
            };
            quote! {
                Schema::Named(
                    Box::new(Named(#name_text.to_string(), #schema))
                )
            }
        }
        _ => panic!("Unsupported type for Nominal schema"),
    }
}
#[proc_macro_attribute]
pub fn hash_discriminator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemEnum);
    
    // Get the original enum
    let original_enum = input.clone();
    
    // Get the name of the enum
    let enum_name = &input.ident;
    
    // Generate names for our hash struct and visitor
    let hashes_struct_name = syn::Ident::new(&format!("{}SchemaHashes", enum_name), enum_name.span());
    let visitor_name = syn::Ident::new(&format!("{}Visitor", enum_name), enum_name.span());
    
    // Collect all variants
    let variants = &input.variants;
    
    // Make sure all variants have a single unnamed field
    for variant in variants {
        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                // This is good - a single unnamed field
            },
            _ => panic!("HashDiscriminator only supports variants with a single unnamed field"),
        }
    }
    
    // Collect all variant names and their field types
    let mut variant_names = Vec::new();
    let mut field_types = Vec::new();
    
    for variant in variants {
        let variant_name = &variant.ident;
        variant_names.push(variant_name);
        
        let field_type = match &variant.fields {
            Fields::Unnamed(fields) => {
                &fields.unnamed.first().unwrap().ty
            },
            _ => unreachable!(), // We've already checked this above
        };
        
        field_types.push(field_type);
    }
    
    // Define fields for our SchemaHashes struct
    let hash_fields = variant_names.iter().map(|variant_name| {
        quote! { pub #variant_name: [u8; 32] }
    });
    
    // Generate initialization for our SchemaHashes struct
    let hash_inits = variant_names.iter().zip(field_types.iter()).map(|(variant_name, field_type)| {
        quote! { 
            #variant_name: *#field_type::schema().stable_hash().as_bytes() 
        }
    });
    
    // Generate serialization arms using the static hashes
    let serialize_arms = variant_names.iter().map(|variant_name| {
        quote! {
            #enum_name::#variant_name(payload) => {
                let hash = SCHEMA_HASHES.get().unwrap().#variant_name;
                
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&hash)?;
                tup.serialize_element(payload)?;
                tup.end()
            }
        }
    });
    
    // Generate deserialization branches using the static hashes
    let deserialize_branches = variant_names.iter().zip(field_types.iter()).map(|(variant_name, field_type)| {
        quote! {
            if hash_bytes == SCHEMA_HASHES.get().unwrap().#variant_name {
                let payload = seq.next_element::<#field_type>()?.ok_or_else(|| 
                    serde::de::Error::custom("missing payload"))?;
                return Ok(#enum_name::#variant_name(payload));
            }
        }
    });
    
    // Generate the implementation
    let generated_impls = quote! {
        // The original enum definition
        #original_enum
        
        // Define a struct to hold the schema hashes
        struct #hashes_struct_name {
            #(#hash_fields),*
        }
        
        impl #hashes_struct_name {
            // Create a new instance with all the hashes computed
            fn new() -> Self {
                Self {
                    #(#hash_inits),*
                }
            }
        }
        
        // Create a static instance of our hashes using std::sync::OnceLock
        use std::sync::OnceLock;
        static SCHEMA_HASHES: OnceLock<#hashes_struct_name> = OnceLock::new();
        
        // Helper function to ensure the hashes are initialized
        fn get_schema_hashes() -> &'static #hashes_struct_name {
            SCHEMA_HASHES.get_or_init(|| #hashes_struct_name::new())
        }
        
        // Implementation of serde::Serialize for the enum
        impl serde::Serialize for #enum_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeTuple;
                let hashes = get_schema_hashes();
                
                match self {
                    #(#enum_name::#variant_names(payload) => {
                        let hash = hashes.#variant_names;
                        
                        let mut tup = serializer.serialize_tuple(2)?;
                        tup.serialize_element(&hash)?;
                        tup.serialize_element(payload)?;
                        tup.end()
                    }),*
                }
            }
        }
        
        // Visitor struct for deserialization
        struct #visitor_name;
        
        impl<'de> serde::de::Visitor<'de> for #visitor_name {
            type Value = #enum_name;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a tuple with a hash discriminator and payload")
            }
            
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                // Deserialize the hash discriminator (first element)
                let hash_bytes: [u8; 32] = seq.next_element()?.ok_or_else(|| 
                    serde::de::Error::custom("missing hash"))?;
                
                // Get the schema hashes
                let hashes = get_schema_hashes();
                
                // Check against our static hashes
                #(
                    if hash_bytes == hashes.#variant_names {
                        let payload = seq.next_element::<#field_types>()?.ok_or_else(|| 
                            serde::de::Error::custom("missing payload"))?;
                        return Ok(#enum_name::#variant_names(payload));
                    }
                )*
                
                // If none matched, return an error
                Err(serde::de::Error::custom("unknown discriminator"))
            }
        }
        
        // Implementation of serde::Deserialize for the enum
        impl<'de> serde::Deserialize<'de> for #enum_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_seq(#visitor_name)
            }
        }
    };

    eprintln!("{}", generated_impls);

    // Return the generated code
    TokenStream::from(generated_impls)
}