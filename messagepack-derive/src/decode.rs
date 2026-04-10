use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive_decode(mut input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let body = match &input.data {
        Data::Struct(data_struct) => {
            quote! { todo!() }
        }
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Decode derive is not yet supported for enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Decode derive is not supported for unions",
            ));
        }
    };

    let de_lifetime: syn::Lifetime = syn::parse_quote!('__msgpack_de);

    {
        input.generics.params.insert(
            0,
            syn::GenericParam::Lifetime(syn::LifetimeParam::new(de_lifetime.clone())),
        );

        let user_lifetimes = input.generics.lifetimes().cloned().collect::<Vec<_>>();
        let where_clause = input.generics.make_where_clause();
        for user_life in user_lifetimes {
            where_clause.predicates.push(syn::parse_quote! {
                #de_lifetime : #user_life
            });
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::messagepack_core::decode::DecodeBorrowed<#de_lifetime> for #name #ty_generics
            #where_clause
        {
            type Value = #name #ty_generics;

            fn decode_borrowed_with_format<__R>(
                __format: ::messagepack_core::Format,
                __reader: &mut __R,
            ) -> ::core::result::Result<Self::Value, ::messagepack_core::decode::Error<__R::Error>>
            where
                __R: ::messagepack_core::io::IoRead<'__de>,
            {
                #body
            }
        }
    })
}
