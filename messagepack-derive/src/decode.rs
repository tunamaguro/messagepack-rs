use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::punctuated::Punctuated;

use crate::shared::{
    ContainerMode, DeriveKind, FieldInfo, StructStyle, add_type_bound, box_inner,
    collect_bound_types, decode_lifetime, option_inner, parse_struct, replace_lifetimes,
};

pub fn derive_decode(input: DeriveInput) -> syn::Result<TokenStream> {
    let info = parse_struct(input, DeriveKind::Decode)?;
    let name = &info.ident;

    let de_lifetime = decode_lifetime();
    let original_generics = info.generics.clone();
    let output_ty_args = original_generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Lifetime(_) => quote! { #de_lifetime },
            syn::GenericParam::Type(tp) => {
                let ident = &tp.ident;
                quote! { #ident }
            }
            syn::GenericParam::Const(cp) => {
                let ident = &cp.ident;
                quote! { #ident }
            }
        })
        .collect::<Vec<_>>();
    let output_ty = if output_ty_args.is_empty() {
        quote! { #name }
    } else {
        quote! { #name :: <#(#output_ty_args),*> }
    };
    let mut generics = original_generics.clone();
    generics.params = generics
        .params
        .into_iter()
        .filter(|param| !matches!(param, syn::GenericParam::Lifetime(_)))
        .collect::<Punctuated<_, syn::token::Comma>>();
    generics.params.insert(0, syn::parse_quote!(#de_lifetime));
    for type_param in original_generics.type_params() {
        let ident = &type_param.ident;
        generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote!(#ident: #de_lifetime));
    }

    add_decode_bounds(&mut generics, &info.style, &de_lifetime);

    let body = match &info.style {
        StructStyle::Unit => decode_unit(),
        StructStyle::Tuple(fields) => {
            decode_tuple(fields, &de_lifetime, info.container.mode, &output_ty)?
        }
        StructStyle::Named(fields) => {
            decode_named(fields, &de_lifetime, info.container.mode, &output_ty)?
        }
    };

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::messagepack_core::decode::DecodeBorrowed<#de_lifetime> for #output_ty
            #where_clause
        {
            type Value = #output_ty;

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

fn decode_unit() -> TokenStream {
    quote! {
        match __format {
            ::messagepack_core::Format::Nil => Ok(Self),
            _ => Err(::messagepack_core::decode::Error::UnexpectedFormat),
        }
    }
}

fn decode_tuple(
    fields: &[FieldInfo],
    de_lifetime: &syn::Lifetime,
    mode: Option<ContainerMode>,
    output_ty: &TokenStream,
) -> syn::Result<TokenStream> {
    if matches!(mode, Some(ContainerMode::Map)) {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "tuple structs cannot use `#[msgpack(map)]`",
        ));
    }
    validate_decode_fields(fields)?;

    let active = fields
        .iter()
        .filter(|field| !field.is_skipped_for_decode())
        .collect::<Vec<_>>();
    let len = active.len();
    let min_len = minimum_array_len(&active);
    let declarations = active
        .iter()
        .map(|field| {
            let local = field_local(field);
            let ty = replace_lifetimes(&field.ty, de_lifetime);
            Ok(quote! {
                let mut #local: ::core::option::Option<#ty> = ::core::option::Option::None;
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let arms = active
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let local = field_local(field);
            let expr = decode_field_expr(field, de_lifetime)?;
            Ok(quote! {
                #index => {
                    if #local.is_some() {
                        return Err(::messagepack_core::decode::Error::InvalidData);
                    }
                    #local = Some(#expr);
                }
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let build = fields.iter().map(tuple_field_build).collect::<Vec<_>>();

    Ok(quote! {
        let __len = match __format {
            ::messagepack_core::Format::FixArray(__len) => usize::from(__len),
            ::messagepack_core::Format::Array16 => ::messagepack_core::decode::NbyteReader::<2>::read(__reader)?,
            ::messagepack_core::Format::Array32 => ::messagepack_core::decode::NbyteReader::<4>::read(__reader)?,
            _ => return Err(::messagepack_core::decode::Error::UnexpectedFormat),
        };
        if !(#min_len..=#len).contains(&__len) {
            return Err(::messagepack_core::decode::Error::InvalidData);
        }
        #(
            #declarations
        )*
        for __index in 0..__len {
            match __index {
                #(
                    #arms
                )*
                _ => unreachable!(),
            }
        }
        Ok(#output_ty(
            #(#build),*
        ))
    })
}

fn decode_named(
    fields: &[FieldInfo],
    de_lifetime: &syn::Lifetime,
    mode: Option<ContainerMode>,
    output_ty: &TokenStream,
) -> syn::Result<TokenStream> {
    validate_decode_fields(fields)?;
    let map_body = decode_named_map(fields, de_lifetime, output_ty)?;
    let array_body = decode_named_array(fields, de_lifetime, mode, output_ty)?;

    Ok(quote! {
        enum FormatKind {
            Map(usize),
            Array(usize),
        }

        let __kind = match __format {
            ::messagepack_core::Format::FixMap(__len) => FormatKind::Map(usize::from(__len)),
            ::messagepack_core::Format::FixArray(__len) => FormatKind::Array(usize::from(__len)),
            ::messagepack_core::Format::Map16 => {
                let __len = ::messagepack_core::decode::NbyteReader::<2>::read(__reader)?;
                FormatKind::Map(__len)
            }
            ::messagepack_core::Format::Map32 => {
                let __len = ::messagepack_core::decode::NbyteReader::<4>::read(__reader)?;
                FormatKind::Map(__len)
            }
            ::messagepack_core::Format::Array16 => {
                let __len = ::messagepack_core::decode::NbyteReader::<2>::read(__reader)?;
                FormatKind::Array(__len)
            }
            ::messagepack_core::Format::Array32 => {
                let __len = ::messagepack_core::decode::NbyteReader::<4>::read(__reader)?;
                FormatKind::Array(__len)
            }
            _ => return Err(::messagepack_core::decode::Error::UnexpectedFormat),
        };

        match __kind {
            FormatKind::Map(__len) => {
                #map_body
            }
            FormatKind::Array(__len) => {
                #array_body
            }
        }
    })
}

fn decode_named_map(
    fields: &[FieldInfo],
    de_lifetime: &syn::Lifetime,
    output_ty: &TokenStream,
) -> syn::Result<TokenStream> {
    let active = fields
        .iter()
        .filter(|field| !field.is_skipped_for_decode())
        .collect::<Vec<_>>();
    let declarations = active
        .iter()
        .map(|field| {
            let local = field_local(field);
            Ok(quote! {
                let mut #local = ::core::option::Option::None;
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let arms = active
        .iter()
        .map(|field| {
            let local = field_local(field);
            let key = field
                .key_name
                .as_ref()
                .expect("named fields always have map keys");
            let key = syn::LitByteStr::new(key.as_bytes(), field.span);
            let decode_expr = decode_field_expr(field, de_lifetime)?;
            Ok(quote! {
                #key => {
                    if #local.is_some() {
                        return Err(::messagepack_core::decode::Error::InvalidData);
                    }
                    #local = Some(#decode_expr);
                }
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let build = fields.iter().map(named_field_build).collect::<Vec<_>>();

    Ok(quote! {
        #(
            #declarations
        )*
        for _ in 0..__len {
            let __key = <::messagepack_core::decode::ReferenceStrBinDecoder as ::messagepack_core::decode::Decode<#de_lifetime>>::decode(__reader)?;
            match __key.as_bytes() {
                #(
                    #arms
                )*
                _ => {
                    let _ = <::messagepack_core::decode::Any<#de_lifetime> as ::messagepack_core::decode::Decode<#de_lifetime>>::decode(__reader)?;
                }
            }
        }
        Ok(#output_ty {
            #(#build),*
        })
    })
}

fn decode_named_array(
    fields: &[FieldInfo],
    de_lifetime: &syn::Lifetime,
    mode: Option<ContainerMode>,
    output_ty: &TokenStream,
) -> syn::Result<TokenStream> {
    let active = match mode.unwrap_or(ContainerMode::Map) {
        ContainerMode::Map => fields
            .iter()
            .filter(|field| !field.is_skipped_for_decode())
            .collect::<Vec<_>>(),
        ContainerMode::Array => sorted_array_fields(fields)?,
    };
    let len = active.len();
    let min_len = minimum_array_len(&active);
    if min_len == len {
        let assignments = active
            .iter()
            .map(|field| {
                let local = field_local(field);
                let decode_expr = decode_field_expr(field, de_lifetime)?;
                Ok(quote! {
                    let #local = #decode_expr;
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;
        let build = fields
            .iter()
            .map(named_field_build_direct)
            .collect::<Vec<_>>();

        return Ok(quote! {
            if __len != #len {
                return Err(::messagepack_core::decode::Error::InvalidData);
            }
            #(
                #assignments
            )*
            Ok(#output_ty {
                #(#build),*
            })
        });
    }

    let declarations = active
        .iter()
        .map(|field| {
            let local = field_local(field);
            Ok(quote! {
                let mut #local = ::core::option::Option::None;
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let assignments = active
        .iter()
        .map(|field| {
            let decode_expr = decode_field_expr(field, de_lifetime)?;
            Ok(quote! {
                #decode_expr
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let arms = active
        .iter()
        .zip(assignments.iter())
        .enumerate()
        .map(|(index, (field, decode_expr))| {
            let local = field_local(field);
            quote! {
                #index => {
                    if #local.is_some() {
                        return Err(::messagepack_core::decode::Error::InvalidData);
                    }
                    #local = Some(#decode_expr);
                }
            }
        })
        .collect::<Vec<_>>();
    let build = fields.iter().map(named_field_build).collect::<Vec<_>>();

    Ok(quote! {
        if !(#min_len..=#len).contains(&__len) {
            return Err(::messagepack_core::decode::Error::InvalidData);
        }
        #(
            #declarations
        )*
        for __index in 0..__len {
            match __index {
                #(
                    #arms
                )*
                _ => unreachable!(),
            }
        }
        Ok(#output_ty {
            #(#build),*
        })
    })
}

fn validate_decode_fields(fields: &[FieldInfo]) -> syn::Result<()> {
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
    let mut active = fields
        .iter()
        .filter(|field| !field.is_skipped_for_decode())
        .collect::<Vec<_>>();
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

fn minimum_array_len(fields: &[&FieldInfo]) -> usize {
    fields
        .iter()
        .rposition(|field| !field.attrs.default && option_inner(&field.ty).is_none())
        .map(|index| index + 1)
        .unwrap_or(0)
}

fn decode_field_expr(field: &FieldInfo, de_lifetime: &syn::Lifetime) -> syn::Result<TokenStream> {
    let target_ty = &field.ty;
    if let Some(path) = &field.attrs.decode_with {
        return Ok(quote! {{
            let __value: #target_ty = #path(__reader)?;
            __value
        }});
    }
    if field.attrs.bytes {
        let decode_ty = replace_lifetimes(target_ty, de_lifetime);
        return Ok(quote! {{
            let __value: #decode_ty = <#decode_ty as ::messagepack_core::decode::DecodeBytes<#de_lifetime>>::decode_bytes(__reader)?;
            __value
        }});
    }
    if let Some(inner) = option_inner(target_ty) {
        let inner_field = FieldInfo {
            member: field.member.clone(),
            ty: inner.clone(),
            attrs: crate::shared::FieldAttrs {
                key: None,
                bytes: field.attrs.bytes,
                default: false,
                encode_with: None,
                decode_with: None,
            },
            span: field.span,
            name: field.name.clone(),
            key_name: field.key_name.clone(),
            is_phantom: false,
        };
        let inner_decode_ty = replace_lifetimes(&inner, de_lifetime);
        let inner_expr =
            decode_non_option_with_format_expr(&inner_field, de_lifetime, quote!(__format))?;
        return Ok(quote! {{
            let __format = <::messagepack_core::Format as ::messagepack_core::decode::DecodeBorrowed<#de_lifetime>>::decode_borrowed(__reader)?;
            let __value: #target_ty = match __format {
                ::messagepack_core::Format::Nil => ::core::option::Option::None,
                __format => {
                    let __inner: #inner_decode_ty = #inner_expr;
                    ::core::option::Option::Some(__inner)
                }
            };
            __value
        }});
    }
    decode_non_option_expr(field, de_lifetime)
}

fn decode_non_option_expr(
    field: &FieldInfo,
    de_lifetime: &syn::Lifetime,
) -> syn::Result<TokenStream> {
    let target_ty = &field.ty;
    let decode_ty = replace_lifetimes(target_ty, de_lifetime);
    if field.attrs.bytes {
        return Ok(quote! {{
            let __value: #decode_ty = <#decode_ty as ::messagepack_core::decode::DecodeBytes<#de_lifetime>>::decode_bytes(__reader)?;
            __value
        }});
    }
    if let Some(inner) = box_inner(target_ty) {
        let inner_field = FieldInfo {
            member: field.member.clone(),
            ty: inner.clone(),
            attrs: field.attrs.clone(),
            span: field.span,
            name: field.name.clone(),
            key_name: field.key_name.clone(),
            is_phantom: false,
        };
        let inner_expr = decode_field_expr(&inner_field, de_lifetime)?;
        return Ok(quote! {{
            extern crate alloc as __msgpack_alloc;
            let __value: #target_ty = __msgpack_alloc::boxed::Box::new(#inner_expr);
            __value
        }});
    }
    Ok(quote! {{
        let __value: #decode_ty = <#decode_ty as ::messagepack_core::decode::DecodeBorrowed<#de_lifetime>>::decode_borrowed(__reader)?;
        __value
    }})
}

fn decode_non_option_with_format_expr(
    field: &FieldInfo,
    de_lifetime: &syn::Lifetime,
    format: TokenStream,
) -> syn::Result<TokenStream> {
    let target_ty = &field.ty;
    let decode_ty = replace_lifetimes(target_ty, de_lifetime);
    if field.attrs.bytes {
        return Ok(quote! {{
            let __value: #decode_ty = <#decode_ty as ::messagepack_core::decode::DecodeBytes<#de_lifetime>>::decode_bytes_with_format(#format, __reader)?;
            __value
        }});
    }
    if let Some(inner) = box_inner(target_ty) {
        let inner_field = FieldInfo {
            member: field.member.clone(),
            ty: inner.clone(),
            attrs: field.attrs.clone(),
            span: field.span,
            name: field.name.clone(),
            key_name: field.key_name.clone(),
            is_phantom: false,
        };
        let inner_expr = decode_non_option_with_format_expr(&inner_field, de_lifetime, format)?;
        return Ok(quote! {{
            extern crate alloc as __msgpack_alloc;
            let __value: #target_ty = __msgpack_alloc::boxed::Box::new(#inner_expr);
            __value
        }});
    }
    Ok(quote! {{
        let __value: #decode_ty = <#decode_ty as ::messagepack_core::decode::DecodeBorrowed<#de_lifetime>>::decode_borrowed_with_format(#format, __reader)?;
        __value
    }})
}

fn field_local(field: &FieldInfo) -> syn::Ident {
    match &field.member {
        syn::Member::Named(name) => syn::Ident::new(&format!("__{}", name), name.span()),
        syn::Member::Unnamed(index) => syn::Ident::new(
            &format!("__field_{}", index.index),
            proc_macro2::Span::call_site(),
        ),
    }
}

fn tuple_field_build(field: &FieldInfo) -> TokenStream {
    if field.is_phantom {
        return quote! { ::core::default::Default::default() };
    }
    let local = field_local(field);
    if field.attrs.default {
        quote! { #local.unwrap_or_default() }
    } else if option_inner(&field.ty).is_some() {
        quote! { #local.unwrap_or(::core::option::Option::None) }
    } else {
        quote! { #local.ok_or(::messagepack_core::decode::Error::InvalidData)? }
    }
}

fn named_field_build(field: &FieldInfo) -> TokenStream {
    let member = &field.member;
    let value = if field.is_phantom {
        quote! { ::core::default::Default::default() }
    } else {
        let local = field_local(field);
        if field.attrs.default {
            quote! { #local.unwrap_or_default() }
        } else if option_inner(&field.ty).is_some() {
            quote! { #local.unwrap_or(::core::option::Option::None) }
        } else {
            quote! { #local.ok_or(::messagepack_core::decode::Error::InvalidData)? }
        }
    };
    quote! { #member: #value }
}

fn named_field_build_direct(field: &FieldInfo) -> TokenStream {
    let member = &field.member;
    let value = if field.is_phantom {
        quote! { ::core::default::Default::default() }
    } else {
        let local = field_local(field);
        quote! { #local }
    };
    quote! { #member: #value }
}

fn add_decode_bounds(
    generics: &mut syn::Generics,
    style: &StructStyle,
    de_lifetime: &syn::Lifetime,
) {
    let borrowed_bound_for = |ty: &syn::Type| -> syn::TypeParamBound {
        syn::parse_quote!(::messagepack_core::decode::DecodeBorrowed<#de_lifetime, Value = #ty>)
    };
    let bytes_bound: syn::TypeParamBound =
        syn::parse_quote!(::messagepack_core::decode::DecodeBytes<#de_lifetime>);
    let default_bound: syn::TypeParamBound = syn::parse_quote!(::core::default::Default);

    let fields = match style {
        StructStyle::Named(fields) | StructStyle::Tuple(fields) => fields,
        StructStyle::Unit => return,
    };

    for field in fields {
        if field.is_phantom {
            continue;
        }
        if field.attrs.default {
            add_type_bound(generics, field.ty.clone(), default_bound.clone());
        }
        if field.attrs.decode_with.is_some() {
            continue;
        }
        if field.attrs.bytes {
            let decode_ty = replace_lifetimes(&field.ty, de_lifetime);
            add_type_bound(generics, decode_ty, bytes_bound.clone());
        } else {
            for ty in collect_bound_types(&field.ty, generics) {
                let decode_ty = replace_lifetimes(&ty, de_lifetime);
                add_type_bound(generics, decode_ty.clone(), borrowed_bound_for(&decode_ty));
            }
        }
    }
}
