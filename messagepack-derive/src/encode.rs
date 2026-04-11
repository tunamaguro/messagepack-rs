use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

use crate::attrs::{ContainerAttrs, FieldAttrs};
use crate::bound;

pub fn derive_encode(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let data_struct = match &input.data {
        Data::Struct(ds) => ds,
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "Encode derive is not yet supported for enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "Encode derive is not supported for unions",
            ));
        }
    };

    // Parse and validate container attributes.
    let container_attrs = ContainerAttrs::from_attrs(&input.attrs)?;

    // Categorize fields for bound generation.
    let mut bound_fields: Vec<&syn::Field> = Vec::new();
    let mut bytes_types: Vec<&syn::Type> = Vec::new();
    let mut default_types: Vec<&syn::Type> = Vec::new();

    for field in &data_struct.fields {
        let fa = FieldAttrs::from_attrs(&field.attrs)?;

        // Validate: array mode requires key on every field.
        if container_attrs.array && fa.key.is_none() {
            return Err(syn::Error::new_spanned(
                field,
                "all fields must have `#[msgpack(key = N)]` when using `#[msgpack(array)]`",
            ));
        }

        if fa.encode_with.is_some() {
            continue;
        }
        if fa.bytes {
            bytes_types.push(&field.ty);
        } else if fa.default {
            default_types.push(&field.ty);
        } else {
            bound_fields.push(field);
        }
    }

    let body = quote! { todo!() };

    let encode_bound: syn::Path = syn::parse_quote!(::messagepack_core::encode::Encode);
    let generics = bound::with_bound(&input.generics, &bound_fields, &encode_bound);

    let as_ref_bound: syn::Path = syn::parse_quote!(::messagepack_core::encode::bin::EncodeBytes);
    let generics = bound::with_type_bound(&generics, &bytes_types, &as_ref_bound);

    let default_bound: syn::Path = syn::parse_quote!(::core::default::Default);
    let generics = bound::with_type_bound(&generics, &default_types, &default_bound);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::messagepack_core::encode::Encode for #name #ty_generics
            #where_clause
        {
            fn encode<__W: ::messagepack_core::io::IoWrite>(&self, writer: &mut __W) -> ::core::result::Result<usize, ::messagepack_core::encode::Error<<__W as ::messagepack_core::io::IoWrite>::Error>> {
                #body
            }
        }
    })
}
