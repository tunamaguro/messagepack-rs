use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

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

    let body = quote! { todo!() };

    let encode_bound: syn::Path = syn::parse_quote!(::messagepack_core::encode::Encode);
    let generics = bound::with_bound(&input.generics, &data_struct.fields, &encode_bound);
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
