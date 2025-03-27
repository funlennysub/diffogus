#![deny(missing_docs)]

//! # diffogus_derive
//!
//! This crate provides a derive macro to help users implement diffing for their types
//!

use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
#[cfg(feature = "serde")]
use quote::ToTokens;
use structmeta::{NameValue, StructMeta};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse, Attribute, Data, DeriveInput, Field, Fields, Ident, Type, Visibility};

/// Diff derive macro
#[proc_macro_derive(Diff, attributes(diff))]
pub fn derive_diff_macro(input: TokenStream) -> TokenStream {
    derive_diff_or_error(input).unwrap_or_else(|err| err.to_compile_error().into())
}

fn filter_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
    attrs.iter().filter(|attr| attr.path().is_ident("diff"))
}

#[derive(StructMeta, Default)]
struct StructAttrs {
    vis: Option<NameValue<Visibility>>,
}

fn derive_diff_or_error(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = parse(input)?;

    let ident = input.ident;
    let tokens = match input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named_fields) => {
                derive_diff_named_structs(ident, &input.attrs, &named_fields.named)?
            }
            _ => todo!("Only structs with named fields are supported right now"),
        },
        _ => todo!("Only structs with named fields are supported right now"),
    }
    .into();

    Ok(tokens)
}

fn derive_diff_named_structs(
    ident: Ident,
    attrs: &[Attribute],
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<TokenStream2> {
    let struct_name = Ident::new(
        &format!("{}DIff", ident.to_string().to_upper_camel_case()),
        Span::call_site(),
    );

    let struct_attrs = filter_attrs(attrs)
        .find_map(|a| a.parse_args::<StructAttrs>().ok())
        .unwrap_or_default();
    let vis = struct_attrs.vis.map(|f| f.value);

    let names: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    #[cfg(feature = "serde")]
    let diff_struct = generate_diff_struct_serde(&struct_name, &vis, &names, &types, fields)?;
    #[cfg(not(feature = "serde"))]
    let diff_struct = generate_diff_struct(&struct_name, &vis, &names, &types)?;
    let diff_impl = generate_diffable_impl(&ident, &struct_name, &names)?;

    Ok(quote! {
        #diff_struct

        #diff_impl
    })
}

fn generate_diffable_impl(
    ident: &Ident,
    struct_name: &Ident,
    names: &Vec<&Option<Ident>>,
) -> syn::Result<TokenStream2> {
    Ok(quote! {
        impl ::diffogus::diff::Diffable for #ident {
            type Repr = #struct_name;

            fn diff(&self, b: &Self) -> Self::Repr {
                #struct_name {
                    #(#names: self.#names.diff(&b.#names)),*
                }
            }
        }

        impl ::diffogus::diff::Changeable for #struct_name {
            fn is_changed(&self) -> bool {
                #(self.#names.is_changed()) || *
            }
        }
    })
}

#[cfg(feature = "serde")]
fn generate_diff_struct_serde(
    struct_name: &Ident,
    vis: &Option<Visibility>,
    names: &Vec<&Option<Ident>>,
    types: &Vec<&Type>,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<TokenStream2> {
    let skips: Vec<_> = fields
        .iter()
        .map(|f| {
            let ty = format!(
                "<<{} as ::diffogus::diff::Diffable>::Repr as ::diffogus::diff::Changeable>::is_unchanged",
                &f.ty.to_token_stream()
            );
            quote! { #[serde(default, skip_serializing_if = #ty)] }
        })
        .collect();

    Ok(quote! {
        #[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
        #vis struct #struct_name {
            #(
                #skips
                #vis #names: <#types as ::diffogus::diff::Diffable>::Repr
            ),*
        }
    })
}

#[cfg(not(feature = "serde"))]
fn generate_diff_struct(
    struct_name: &Ident,
    vis: &Option<Visibility>,
    names: &Vec<&Option<Ident>>,
    types: &Vec<&Type>,
) -> syn::Result<TokenStream2> {
    Ok(quote! {
        #[derive(Default, Debug)]
        #vis struct #struct_name {
            #(
                #vis #names: <#types as ::diffogus::diff::Diffable>::Repr
            ),*
        }
    })
}
