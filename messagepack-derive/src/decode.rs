use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_decode(mut input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let de_lifetime: syn::Lifetime = syn::parse_quote!('__msgpack_de);
    let reader_lifetime: syn::Lifetime = syn::parse_quote!('__reader);

    let user_generics = input.generics.clone();
    let (_, ty_generics, _) = user_generics.split_for_impl();
    let decode_where_lifetimes = user_generics.lifetimes().map(|life| {
        let lifetime = &life.lifetime;
        quote! {
            #lifetime : #reader_lifetime
        }
    });

    // Add decode lifetime to generics and where clause
    input
        .generics
        .params
        .insert(0, syn::parse_quote!(#de_lifetime));

    for lifetime in user_generics.lifetimes().map(|l| &l.lifetime) {
        let where_clause = input.generics.make_where_clause();
        where_clause
            .predicates
            .push(syn::parse_quote!(#de_lifetime : #lifetime));
    }

    let (impl_generics, _, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::messagepack_core::decode::Decode<#de_lifetime> for #name #ty_generics #where_clause
        {
            type Value<#reader_lifetime> = #name #ty_generics
                where
                    Self: #reader_lifetime,
                    #de_lifetime: #reader_lifetime;

            fn decode_with_format<#reader_lifetime, __R>(
                format: ::messagepack_core::Format,
                reader: &#reader_lifetime mut __R,
            ) -> ::core::result::Result<Self::Value<#reader_lifetime>, ::messagepack_core::decode::Error<__R::Error>>
            where
                __R: ::messagepack_core::io::IoRead<#de_lifetime>,
                #de_lifetime: #reader_lifetime,
                #(#decode_where_lifetimes,)*
            {
                todo!()
            }
        }
    })
}
