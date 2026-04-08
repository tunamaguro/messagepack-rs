use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

use crate::attrs::{ContainerAttrs, FieldAttrs};

pub fn derive_encode(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(data_struct) => {
            encode_struct(name, &input.attrs, &data_struct.fields)?
        }
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Encode derive is not yet supported for enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Encode derive is not supported for unions",
            ));
        }
    };

    // Collect type/lifetime/const params for the impl header.
    // Lifetimes must appear before type and const params, so split them.
    let lifetime_params: Vec<_> = input.generics.lifetimes().collect();
    let type_const_params: Vec<_> = input
        .generics
        .params
        .iter()
        .filter(|p| !matches!(p, syn::GenericParam::Lifetime(_)))
        .collect();

    // Build an augmented where clause that requires each generic type parameter
    // to implement `Encode`.
    let mut encode_where = where_clause.cloned().unwrap_or_else(|| syn::parse_quote!(where));
    for param in input.generics.type_params() {
        let ident = &param.ident;
        encode_where
            .predicates
            .push(syn::parse_quote!(#ident: ::messagepack_core::encode::Encode));
    }

    Ok(quote! {
        impl<#(#lifetime_params,)* #(#type_const_params),*> ::messagepack_core::encode::Encode for #name #ty_generics
            #encode_where
        {
            fn encode<__W: ::messagepack_core::io::IoWrite>(&self, writer: &mut __W) -> ::core::result::Result<usize, ::messagepack_core::encode::Error<<__W as ::messagepack_core::io::IoWrite>::Error>> {
                #body
            }
        }
    })
}

fn encode_struct(
    _name: &syn::Ident,
    attrs: &[syn::Attribute],
    fields: &Fields,
) -> syn::Result<TokenStream> {
    let container = ContainerAttrs::from_attrs(attrs)?;

    match fields {
        Fields::Named(named) => {
            let is_map = container.is_map(true);
            if is_map {
                encode_named_as_map(named)
            } else {
                encode_named_as_array(named)
            }
        }
        Fields::Unnamed(unnamed) => encode_tuple_struct(unnamed),
        Fields::Unit => {
            // Unit struct → encode as nil
            Ok(quote! {
                ::messagepack_core::encode::Encode::encode(&(), writer)
            })
        }
    }
}

fn encode_named_as_map(fields: &syn::FieldsNamed) -> syn::Result<TokenStream> {
    let num_fields = fields.named.len();

    let mut field_encoders = Vec::new();
    for field in &fields.named {
        let field_attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        let key_encode = quote! {
            __n += ::messagepack_core::encode::Encode::encode(&#field_name_str, writer)?;
        };

        let value_encode = encode_field_value(field, &field_attrs, quote! { &self.#field_name })?;

        field_encoders.push(quote! {
            #key_encode
            #value_encode
        });
    }

    Ok(quote! {
        let mut __n = 0usize;
        __n += ::messagepack_core::encode::Encode::encode(
            &::messagepack_core::encode::map::MapFormatEncoder::new(#num_fields),
            writer,
        )?;
        #(#field_encoders)*
        Ok(__n)
    })
}

fn encode_named_as_array(fields: &syn::FieldsNamed) -> syn::Result<TokenStream> {
    // Collect fields with their key indices.
    let mut indexed_fields: Vec<(usize, &syn::Field)> = Vec::new();
    for field in &fields.named {
        let field_attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let key = field_attrs.key.ok_or_else(|| {
            syn::Error::new_spanned(
                field,
                "all fields must have `#[msgpack(key = N)]` when using `#[msgpack(array)]`",
            )
        })?;
        indexed_fields.push((key, field));
    }

    // Sort by key.
    indexed_fields.sort_by_key(|(k, _)| *k);

    // Validate keys are contiguous 0..N
    for (i, (k, field)) in indexed_fields.iter().enumerate() {
        if *k != i {
            return Err(syn::Error::new_spanned(
                field,
                format!(
                    "array keys must be contiguous starting from 0; expected key {}, found {}",
                    i, k
                ),
            ));
        }
    }

    let num_fields = indexed_fields.len();

    let mut field_encoders = Vec::new();
    for (_key, field) in &indexed_fields {
        let field_attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let field_name = field.ident.as_ref().unwrap();
        let value_encode = encode_field_value(field, &field_attrs, quote! { &self.#field_name })?;
        field_encoders.push(value_encode);
    }

    Ok(quote! {
        let mut __n = 0usize;
        __n += ::messagepack_core::encode::Encode::encode(
            &::messagepack_core::encode::array::ArrayFormatEncoder(#num_fields),
            writer,
        )?;
        #(#field_encoders)*
        Ok(__n)
    })
}

fn encode_tuple_struct(fields: &syn::FieldsUnnamed) -> syn::Result<TokenStream> {
    let num_fields = fields.unnamed.len();

    let mut field_encoders = Vec::new();
    for (i, field) in fields.unnamed.iter().enumerate() {
        let field_attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let idx = syn::Index::from(i);
        let value_encode = encode_field_value(field, &field_attrs, quote! { &self.#idx })?;
        field_encoders.push(value_encode);
    }

    Ok(quote! {
        let mut __n = 0usize;
        __n += ::messagepack_core::encode::Encode::encode(
            &::messagepack_core::encode::array::ArrayFormatEncoder(#num_fields),
            writer,
        )?;
        #(#field_encoders)*
        Ok(__n)
    })
}

fn encode_field_value(
    _field: &syn::Field,
    attrs: &FieldAttrs,
    accessor: TokenStream,
) -> syn::Result<TokenStream> {
    if let Some(ref encode_fn) = attrs.encode_with {
        Ok(quote! {
            __n += #encode_fn(#accessor, writer)?;
        })
    } else if attrs.bytes {
        Ok(quote! {
            __n += ::messagepack_core::encode::Encode::encode(
                &::messagepack_core::encode::bin::BinaryEncoder(#accessor),
                writer,
            )?;
        })
    } else {
        Ok(quote! {
            __n += ::messagepack_core::encode::Encode::encode(#accessor, writer)?;
        })
    }
}
