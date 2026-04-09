use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_encode(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::messagepack_core::encode::Encode for #name #ty_generics
            #where_clause
        {
            fn encode<__W: ::messagepack_core::io::IoWrite>(&self, writer: &mut __W) -> ::core::result::Result<usize, ::messagepack_core::encode::Error<<__W as ::messagepack_core::io::IoWrite>::Error>> {
                todo!()
            }
        }
    })
}
