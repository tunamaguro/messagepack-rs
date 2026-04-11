use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_decode(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let body = quote! { todo!() };

    let de_lifetime: syn::Lifetime = syn::parse_quote!('__msgpack_de);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

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
