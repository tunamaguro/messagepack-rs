use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_decode(mut input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let body = quote! { todo!() };

    let original_generics = input.generics.clone();
    let (_, ty_generics, _) = original_generics.split_for_impl();

    let de_lifetime: syn::Lifetime = syn::parse_quote!('__msgpack_de);
    input
        .generics
        .params
        .insert(0, syn::parse_quote!(#de_lifetime));
    let (impl_generics, _, where_clause) = input.generics.split_for_impl();

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
