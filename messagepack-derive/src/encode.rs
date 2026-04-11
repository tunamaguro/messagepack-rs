use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::shared::{
    ContainerMode, DeriveKind, FieldInfo, StructStyle, add_type_bound, collect_bound_types,
    ensure_where_clause, parse_struct,
};

pub fn derive_encode(input: DeriveInput) -> syn::Result<TokenStream> {
    let info = parse_struct(input, DeriveKind::Encode)?;
    let name = &info.ident;
    let mut generics = info.generics.clone();

    let body = match &info.style {
        StructStyle::Unit => {
            quote! {
                ::messagepack_core::encode::NilEncoder.encode(writer)
            }
        }
        StructStyle::Tuple(fields) => encode_tuple(fields, info.container.mode)?,
        StructStyle::Named(fields) => encode_named(fields, info.container.mode)?,
    };

    add_encode_bounds(&mut generics, &info.style);

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

fn encode_tuple(fields: &[FieldInfo], mode: Option<ContainerMode>) -> syn::Result<TokenStream> {
    if matches!(mode, Some(ContainerMode::Map)) {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "tuple structs cannot use `#[msgpack(map)]`",
        ));
    }

    validate_skipped_fields(fields)?;

    let active = fields.iter().filter(|field| !field.is_skipped()).collect::<Vec<_>>();
    let writes = active
        .iter()
        .map(|field| encode_field_expr(field))
        .collect::<syn::Result<Vec<_>>>()?;
    let len = active.len();

    Ok(quote! {
        let mut __size = 0usize;
        __size += ::messagepack_core::encode::array::ArrayFormatEncoder(#len).encode(writer)?;
        #(
            __size += #writes;
        )*
        Ok(__size)
    })
}

fn encode_named(fields: &[FieldInfo], mode: Option<ContainerMode>) -> syn::Result<TokenStream> {
    validate_skipped_fields(fields)?;

    match mode.unwrap_or(ContainerMode::Map) {
        ContainerMode::Map => {
            let active = fields.iter().filter(|field| !field.is_skipped()).collect::<Vec<_>>();
            let writes = active
                .iter()
                .map(|field| {
                    let key = field
                        .key_name
                        .as_ref()
                        .expect("named fields always have key names");
                    let encode_value = encode_field_expr(field)?;
                    Ok(quote! {
                        __size += ::messagepack_core::encode::Encode::encode(&#key, writer)?;
                        __size += #encode_value;
                    })
                })
                .collect::<syn::Result<Vec<_>>>()?;
            let len = active.len();

            Ok(quote! {
                let mut __size = 0usize;
                __size += ::messagepack_core::encode::map::MapFormatEncoder::new(#len).encode(writer)?;
                #(
                    #writes
                )*
                Ok(__size)
            })
        }
        ContainerMode::Array => {
            let active = sorted_array_fields(fields)?;
            let writes = active
                .iter()
                .map(|field| encode_field_expr(field))
                .collect::<syn::Result<Vec<_>>>()?;
            let len = active.len();

            Ok(quote! {
                let mut __size = 0usize;
                __size += ::messagepack_core::encode::array::ArrayFormatEncoder(#len).encode(writer)?;
                #(
                    __size += #writes;
                )*
                Ok(__size)
            })
        }
    }
}

fn validate_skipped_fields(fields: &[FieldInfo]) -> syn::Result<()> {
    for field in fields {
        if field.is_phantom && field.attrs.key.is_some() {
            return Err(syn::Error::new(
                field.span,
                "PhantomData fields cannot use `#[msgpack(key = N)]`",
            ));
        }
    }
    Ok(())
}

fn sorted_array_fields(fields: &[FieldInfo]) -> syn::Result<Vec<&FieldInfo>> {
    let mut active = fields.iter().filter(|field| !field.is_skipped()).collect::<Vec<_>>();
    for field in &active {
        if field.attrs.key.is_none() {
            return Err(syn::Error::new(
                field.span,
                "all fields must have `#[msgpack(key = N)]` when using `#[msgpack(array)]`",
            ));
        }
    }
    active.sort_by_key(|field| field.attrs.key.expect("checked above"));
    for (expected, field) in active.iter().enumerate() {
        if field.attrs.key != Some(expected) {
            return Err(syn::Error::new(
                field.span,
                "`#[msgpack(array)]` keys must be contiguous starting at 0",
            ));
        }
    }
    Ok(active)
}

fn encode_field_expr(field: &FieldInfo) -> syn::Result<TokenStream> {
    let member = &field.member;
    if let Some(path) = &field.attrs.encode_with {
        return Ok(quote! { #path(&self.#member, writer)? });
    }
    if field.attrs.bytes {
        return Ok(quote! {
            ::messagepack_core::encode::bin::EncodeBytes::encode_bytes(&self.#member, writer)?
        });
    }
    Ok(quote! { ::messagepack_core::encode::Encode::encode(&self.#member, writer)? })
}

fn add_encode_bounds(generics: &mut syn::Generics, style: &StructStyle) {
    let encode_bound: syn::TypeParamBound = syn::parse_quote!(::messagepack_core::encode::Encode);
    let bytes_bound: syn::TypeParamBound =
        syn::parse_quote!(::messagepack_core::encode::bin::EncodeBytes);

    let fields = match style {
        StructStyle::Named(fields) | StructStyle::Tuple(fields) => fields,
        StructStyle::Unit => return,
    };

    ensure_where_clause(generics);
    for field in fields {
        if field.is_skipped() || field.attrs.encode_with.is_some() {
            continue;
        }
        if field.attrs.bytes {
            add_type_bound(generics, field.ty.clone(), bytes_bound.clone());
        } else {
            for ty in collect_bound_types(&field.ty, generics) {
                add_type_bound(generics, ty, encode_bound.clone());
            }
        }
    }
}
