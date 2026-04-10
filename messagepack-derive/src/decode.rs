use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

use crate::bound;

pub fn derive_decode(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let data_struct = match &input.data {
        Data::Struct(ds) => ds,
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "Decode derive is not yet supported for enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "Decode derive is not supported for unions",
            ));
        }
    };

    let body = quote! { todo!() };

    let de_lifetime: syn::Lifetime = syn::parse_quote!('__msgpack_de);

    // ty_generics from original (without the added 'de lifetime)
    let (_, ty_generics, _) = input.generics.split_for_impl();

    // Build impl generics: add trait bounds, then prepend 'de lifetime
    let decode_bound: syn::Path =
        syn::parse_quote!(::messagepack_core::decode::DecodeBorrowed<#de_lifetime>);
    let generics = bound::with_bound(&input.generics, &data_struct.fields, &decode_bound);
    let generics = bound::with_de_lifetime(&generics, &de_lifetime);
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::messagepack_core::decode::DecodeBorrowed<#de_lifetime> for #name #ty_generics
            #where_clause
        {
            type Value = #name #ty_generics;

            fn decode_borrowed_with_format<__R>(
                __format: ::messagepack_core::Format,
                __reader: &mut __R,
            ) -> ::core::result::Result<Self::Value, ::messagepack_core::decode::Error<__R::Error>>
            where
                __R: ::messagepack_core::io::IoRead<#de_lifetime>,
            {
                #body
            }
        }
    })
}
