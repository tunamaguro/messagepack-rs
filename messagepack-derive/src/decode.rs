use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, WherePredicate};

use crate::attrs::{ContainerAttrs, FieldAttrs};
use crate::types::{
    box_inner_type, collect_dependent_types, field_is_skipped_on_wire, option_inner_type,
    replace_lifetimes_in_type, type_is_option, type_is_phantom_data,
};

pub fn derive_decode(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (_, _ty_generics, where_clause) = input.generics.split_for_impl();

    // Collect user-defined lifetime names so we can replace them with '__de.
    let user_lifetimes: HashSet<String> = input
        .generics
        .lifetimes()
        .map(|lt| lt.lifetime.ident.to_string())
        .collect();
    let type_param_idents: HashSet<String> = input
        .generics
        .type_params()
        .map(|param| param.ident.to_string())
        .collect();

    let body = match &input.data {
        Data::Struct(data_struct) => {
            decode_struct(name, &input.attrs, &data_struct.fields, &user_lifetimes)?
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

    // Collect only non-lifetime params for the impl header.
    let type_const_params: Vec<_> = input
        .generics
        .params
        .iter()
        .filter(|p| !matches!(p, syn::GenericParam::Lifetime(_)))
        .collect();

    // Build ty generics that replace every user lifetime with a synthetic lifetime.
    let de_lifetime: syn::Lifetime = syn::parse_quote!('__de);
    let de_ty_generic_args: Vec<TokenStream> = input
        .generics
        .params
        .iter()
        .map(|p| match p {
            syn::GenericParam::Lifetime(_) => quote! { #de_lifetime },
            syn::GenericParam::Type(t) => {
                let ident = &t.ident;
                quote! { #ident }
            }
            syn::GenericParam::Const(c) => {
                let ident = &c.ident;
                quote! { #ident }
            }
        })
        .collect();
    let de_ty_generics = if de_ty_generic_args.is_empty() {
        quote! {}
    } else {
        quote! { <#(#de_ty_generic_args),*> }
    };

    // Build augmented where clause requiring each decoded field type to
    // implement the trait the generated code uses.
    let mut decode_where = where_clause
        .cloned()
        .unwrap_or_else(|| syn::parse_quote!(where));
    for predicate in collect_decode_bounds(&input.data, &user_lifetimes, &type_param_idents)? {
        decode_where.predicates.push(predicate);
    }

    Ok(quote! {
        impl<'__de, #(#type_const_params),*> ::messagepack_core::decode::DecodeBorrowed<'__de> for #name #de_ty_generics
            #decode_where
        {
            type Value = #name #de_ty_generics;

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

fn decode_struct(
    name: &syn::Ident,
    attrs: &[syn::Attribute],
    fields: &Fields,
    user_lifetimes: &HashSet<String>,
) -> syn::Result<TokenStream> {
    let container = ContainerAttrs::from_attrs(attrs)?;

    match fields {
        Fields::Named(named) => {
            let is_map = container.is_map(true);
            if is_map {
                // Default map mode: decode accepts both map and array
                decode_named_fields(name, named, true, user_lifetimes)
            } else {
                // Array mode: decode accepts both map and array
                decode_named_fields(name, named, false, user_lifetimes)
            }
        }
        Fields::Unnamed(unnamed) => decode_tuple_struct(name, unnamed, user_lifetimes),
        Fields::Unit => Ok(quote! {
            <() as ::messagepack_core::decode::DecodeBorrowed<'__de>>::decode_borrowed_with_format(__format, __reader)?;
            Ok(#name)
        }),
    }
}

fn collect_decode_bounds(
    data: &Data,
    user_lifetimes: &HashSet<String>,
    type_param_idents: &HashSet<String>,
) -> syn::Result<Vec<WherePredicate>> {
    let mut predicates = Vec::new();

    let fields = match data {
        Data::Struct(data_struct) => &data_struct.fields,
        Data::Enum(_) | Data::Union(_) => return Ok(predicates),
    };

    for field in fields {
        let attrs = FieldAttrs::from_attrs(&field.attrs)?;
        if attrs.decode_with.is_some() {
            continue;
        }

        let decode_ty = replace_lifetimes_in_type(&field.ty, user_lifetimes);
        if attrs.bytes {
            predicates.push(
                syn::parse_quote!(#decode_ty: ::messagepack_core::decode::DecodeBytes<'__de>),
            );
        } else {
            add_decode_type_bounds(&field.ty, user_lifetimes, type_param_idents, &mut predicates);
        }

        if attrs.default || type_is_option(&field.ty) {
            predicates.push(syn::parse_quote!(#decode_ty: ::core::default::Default));
        }
    }

    Ok(predicates)
}

/// Decode named fields; accepts both MessagePack map and array formats.
fn decode_named_fields(
    name: &syn::Ident,
    fields: &syn::FieldsNamed,
    is_map_default: bool,
    user_lifetimes: &HashSet<String>,
) -> syn::Result<TokenStream> {
    // Collect field info
    struct FieldInfo<'a> {
        ident: &'a syn::Ident,
        name_str: String,
        attrs: FieldAttrs,
        ty: &'a syn::Type,
        array_key: usize,
        allow_missing: bool,
        skip_on_wire: bool,
    }

    let mut field_infos: Vec<FieldInfo> = Vec::new();
    for (i, field) in fields.named.iter().enumerate() {
        let ident = field.ident.as_ref().unwrap();
        let attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let skip_on_wire = field_is_skipped_on_wire(field);
        let array_key = if !is_map_default {
            attrs.key.ok_or_else(|| {
                syn::Error::new_spanned(
                    field,
                    "all fields must have `#[msgpack(key = N)]` when using `#[msgpack(array)]`",
                )
            })?
        } else {
            attrs.key.unwrap_or(i)
        };
        field_infos.push(FieldInfo {
            ident,
            name_str: ident.to_string(),
            allow_missing: skip_on_wire || attrs.default || type_is_option(&field.ty),
            attrs,
            ty: &field.ty,
            array_key,
            skip_on_wire,
        });
    }

    // Validate array keys for array mode
    if !is_map_default {
        let mut sorted: Vec<usize> = field_infos.iter().map(|f| f.array_key).collect();
        sorted.sort();
        for (i, k) in sorted.iter().enumerate() {
            if *k != i {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!(
                        "array keys must be contiguous starting from 0; expected {}, found {}",
                        i, k
                    ),
                ));
            }
        }
    }

    // Generate Option variable declarations for each field.
    let field_option_decls: Vec<TokenStream> = field_infos
        .iter()
        .filter(|fi| !fi.skip_on_wire)
        .map(|fi| {
            let var = format_ident!("__field_{}", fi.ident);
            let output_ty = replace_lifetimes_in_type(fi.ty, user_lifetimes);
            quote! {
                let mut #var: ::core::option::Option<#output_ty> = ::core::option::Option::None;
            }
        })
        .collect();

    // Generate map-branch match arms: match on field name string.
    let map_match_arms: Vec<TokenStream> = field_infos
        .iter()
        .filter(|fi| !fi.skip_on_wire)
        .map(|fi| {
            let var = format_ident!("__field_{}", fi.ident);
            let name_str = &fi.name_str;
            let decode_expr = decode_field_expr(fi.ty, &fi.attrs, user_lifetimes);
            quote! {
                #name_str => {
                    #var = ::core::option::Option::Some(#decode_expr);
                }
            }
        })
        .collect();

    // Generate array-branch decode in sorted key order.
    let mut sorted_by_key: Vec<&FieldInfo> = field_infos.iter().collect();
    sorted_by_key.sort_by_key(|fi| fi.array_key);

    let array_decode_stmts: Vec<TokenStream> = sorted_by_key
        .iter()
        .filter(|fi| !fi.skip_on_wire)
        .map(|fi| {
            let var = format_ident!("__field_{}", fi.ident);
            let decode_expr = decode_field_expr(fi.ty, &fi.attrs, user_lifetimes);
            quote! {
                #var = ::core::option::Option::Some(#decode_expr);
            }
        })
        .collect();

    let missing_allowed_count = field_infos
        .iter()
        .filter(|fi| !fi.skip_on_wire && fi.allow_missing)
        .count();
    let num_wire_fields = field_infos.iter().filter(|fi| !fi.skip_on_wire).count();

    // Generate constructor: unwrap each Option.
    let constructor_fields: Vec<TokenStream> = field_infos
        .iter()
        .map(|fi| {
            let ident = fi.ident;
            let var = format_ident!("__field_{}", fi.ident);
            if fi.skip_on_wire {
                quote! {
                    #ident: ::core::marker::PhantomData
                }
            } else if fi.allow_missing {
                quote! {
                    #ident: #var.unwrap_or_default()
                }
            } else {
                quote! {
                    #ident: #var.ok_or(::messagepack_core::decode::Error::InvalidData)?
                }
            }
        })
        .collect();

    Ok(quote! {
        match __format {
            ::messagepack_core::Format::FixMap(_)
            | ::messagepack_core::Format::Map16
            | ::messagepack_core::Format::Map32 => {
                let __len = match __format {
                    ::messagepack_core::Format::FixMap(n) => n as usize,
                    ::messagepack_core::Format::Map16 => {
                        let __b = __reader.read_slice(2).map_err(::messagepack_core::decode::Error::Io)?;
                        let __arr: [u8; 2] = __b.as_bytes().try_into().map_err(|_| ::messagepack_core::decode::Error::UnexpectedEof)?;
                        u16::from_be_bytes(__arr) as usize
                    }
                    ::messagepack_core::Format::Map32 => {
                        let __b = __reader.read_slice(4).map_err(::messagepack_core::decode::Error::Io)?;
                        let __arr: [u8; 4] = __b.as_bytes().try_into().map_err(|_| ::messagepack_core::decode::Error::UnexpectedEof)?;
                        u32::from_be_bytes(__arr) as usize
                    }
                    _ => unreachable!(),
                };

                #(#field_option_decls)*

                for _ in 0..__len {
                    // Decode key as str
                    let __key_format = <::messagepack_core::Format as ::messagepack_core::decode::DecodeBorrowed<'__de>>::decode_borrowed(__reader)?;
                    let __key_len = match __key_format {
                        ::messagepack_core::Format::FixStr(n) => n as usize,
                        ::messagepack_core::Format::Str8 => {
                            let __b = __reader.read_slice(1).map_err(::messagepack_core::decode::Error::Io)?;
                            __b.as_bytes()[0] as usize
                        }
                        ::messagepack_core::Format::Str16 => {
                            let __b = __reader.read_slice(2).map_err(::messagepack_core::decode::Error::Io)?;
                            let __arr: [u8; 2] = __b.as_bytes().try_into().map_err(|_| ::messagepack_core::decode::Error::UnexpectedEof)?;
                            u16::from_be_bytes(__arr) as usize
                        }
                        ::messagepack_core::Format::Str32 => {
                            let __b = __reader.read_slice(4).map_err(::messagepack_core::decode::Error::Io)?;
                            let __arr: [u8; 4] = __b.as_bytes().try_into().map_err(|_| ::messagepack_core::decode::Error::UnexpectedEof)?;
                            u32::from_be_bytes(__arr) as usize
                        }
                        _ => return ::core::result::Result::Err(::messagepack_core::decode::Error::UnexpectedFormat),
                    };
                    let __key_data = __reader.read_slice(__key_len).map_err(::messagepack_core::decode::Error::Io)?;
                    let __key_str = ::core::str::from_utf8(__key_data.as_bytes()).map_err(|_| ::messagepack_core::decode::Error::InvalidData)?;

                    match __key_str {
                        #(#map_match_arms)*
                        _ => {
                            // Skip unknown field value
                            <::messagepack_core::decode::Any<'__de> as ::messagepack_core::decode::DecodeBorrowed<'__de>>::decode_borrowed(__reader)?;
                        }
                    }
                }

                Ok(#name {
                    #(#constructor_fields,)*
                })
            }
            ::messagepack_core::Format::FixArray(_)
            | ::messagepack_core::Format::Array16
            | ::messagepack_core::Format::Array32 => {
                let __len = match __format {
                    ::messagepack_core::Format::FixArray(n) => n as usize,
                    ::messagepack_core::Format::Array16 => {
                        let __b = __reader.read_slice(2).map_err(::messagepack_core::decode::Error::Io)?;
                        let __arr: [u8; 2] = __b.as_bytes().try_into().map_err(|_| ::messagepack_core::decode::Error::UnexpectedEof)?;
                        u16::from_be_bytes(__arr) as usize
                    }
                    ::messagepack_core::Format::Array32 => {
                        let __b = __reader.read_slice(4).map_err(::messagepack_core::decode::Error::Io)?;
                        let __arr: [u8; 4] = __b.as_bytes().try_into().map_err(|_| ::messagepack_core::decode::Error::UnexpectedEof)?;
                        u32::from_be_bytes(__arr) as usize
                    }
                    _ => unreachable!(),
                };

                let __required_fields = #num_wire_fields - #missing_allowed_count;
                if __len < __required_fields || __len > #num_wire_fields {
                    return ::core::result::Result::Err(::messagepack_core::decode::Error::InvalidData);
                }

                #(#field_option_decls)*
                #(#array_decode_stmts)*

                Ok(#name {
                    #(#constructor_fields,)*
                })
            }
            _ => ::core::result::Result::Err(::messagepack_core::decode::Error::UnexpectedFormat),
        }
    })
}

fn decode_tuple_struct(
    name: &syn::Ident,
    fields: &syn::FieldsUnnamed,
    user_lifetimes: &HashSet<String>,
) -> syn::Result<TokenStream> {
    let num_fields = fields
        .unnamed
        .iter()
        .filter(|field| !field_is_skipped_on_wire(field))
        .count();

    let mut decode_stmts = Vec::new();
    let mut field_vars = Vec::new();
    for (i, field) in fields.unnamed.iter().enumerate() {
        if field_is_skipped_on_wire(field) {
            field_vars.push(quote! { ::core::marker::PhantomData });
        } else {
            let field_attrs = FieldAttrs::from_attrs(&field.attrs)?;
            let var = format_ident!("__field_{}", i);
            let decode_expr = decode_field_expr(&field.ty, &field_attrs, user_lifetimes);
            let output_ty = replace_lifetimes_in_type(&field.ty, user_lifetimes);
            decode_stmts.push(quote! {
                let #var: #output_ty = #decode_expr;
            });
            field_vars.push(quote! { #var });
        }
    }

    Ok(quote! {
        let __len = match __format {
            ::messagepack_core::Format::FixArray(n) => n as usize,
            ::messagepack_core::Format::Array16 => {
                let __b = __reader.read_slice(2).map_err(::messagepack_core::decode::Error::Io)?;
                let __arr: [u8; 2] = __b.as_bytes().try_into().map_err(|_| ::messagepack_core::decode::Error::UnexpectedEof)?;
                u16::from_be_bytes(__arr) as usize
            }
            ::messagepack_core::Format::Array32 => {
                let __b = __reader.read_slice(4).map_err(::messagepack_core::decode::Error::Io)?;
                let __arr: [u8; 4] = __b.as_bytes().try_into().map_err(|_| ::messagepack_core::decode::Error::UnexpectedEof)?;
                u32::from_be_bytes(__arr) as usize
            }
            _ => return ::core::result::Result::Err(::messagepack_core::decode::Error::UnexpectedFormat),
        };

        if __len != #num_fields {
            return ::core::result::Result::Err(::messagepack_core::decode::Error::InvalidData);
        }

        #(#decode_stmts)*

        Ok(#name(#(#field_vars),*))
    })
}

fn decode_field_expr(
    ty: &syn::Type,
    attrs: &FieldAttrs,
    user_lifetimes: &HashSet<String>,
) -> TokenStream {
    if let Some(ref decode_fn) = attrs.decode_with {
        quote! {
            #decode_fn(__reader)?
        }
    } else if attrs.bytes {
        let replaced_ty = replace_lifetimes_in_type(ty, user_lifetimes);
        quote! {
            <#replaced_ty as ::messagepack_core::decode::DecodeBytes<'__de>>::decode_bytes(__reader)?
        }
    } else {
        decode_type_expr(ty, user_lifetimes)
    }
}

fn add_decode_type_bounds(
    ty: &syn::Type,
    user_lifetimes: &HashSet<String>,
    type_param_idents: &HashSet<String>,
    predicates: &mut Vec<WherePredicate>,
) {
    let mut dependent_types = Vec::new();
    collect_dependent_types(ty, type_param_idents, &mut dependent_types);

    for dependent_ty in dependent_types {
        let decode_ty = replace_lifetimes_in_type(&dependent_ty, user_lifetimes);
        predicates.push(syn::parse_quote!(
            #decode_ty: ::messagepack_core::decode::DecodeBorrowed<'__de, Value = #decode_ty>
        ));
    }
}

fn decode_type_expr(ty: &syn::Type, user_lifetimes: &HashSet<String>) -> TokenStream {
    if let Some(inner_ty) = option_inner_type(ty) {
        let some_decode = decode_type_with_format_expr(inner_ty, user_lifetimes, quote! { __other });
        quote! {{
            let __field_format =
                <::messagepack_core::Format as ::messagepack_core::decode::DecodeBorrowed<'__de>>::decode_borrowed(__reader)?;
            match __field_format {
                ::messagepack_core::Format::Nil => ::core::option::Option::None,
                __other => ::core::option::Option::Some(#some_decode),
            }
        }}
    } else if let Some(inner_ty) = box_inner_type(ty) {
        let inner_decode = decode_type_expr(inner_ty, user_lifetimes);
        quote! {
            Box::new(#inner_decode)
        }
    } else if type_is_phantom_data(ty) {
        quote! {{
            <() as ::messagepack_core::decode::DecodeBorrowed<'__de>>::decode_borrowed(__reader)?;
            ::core::marker::PhantomData
        }}
    } else {
        let replaced_ty = replace_lifetimes_in_type(ty, user_lifetimes);
        quote! {
            <#replaced_ty as ::messagepack_core::decode::DecodeBorrowed<'__de>>::decode_borrowed(__reader)?
        }
    }
}

fn decode_type_with_format_expr(
    ty: &syn::Type,
    user_lifetimes: &HashSet<String>,
    format: TokenStream,
) -> TokenStream {
    if let Some(inner_ty) = option_inner_type(ty) {
        let some_decode =
            decode_type_with_format_expr(inner_ty, user_lifetimes, quote! { __other });
        quote! {{
            match #format {
                ::messagepack_core::Format::Nil => ::core::option::Option::None,
                __other => ::core::option::Option::Some(#some_decode),
            }
        }}
    } else if let Some(inner_ty) = box_inner_type(ty) {
        let inner_decode = decode_type_with_format_expr(inner_ty, user_lifetimes, format);
        quote! {
            Box::new(#inner_decode)
        }
    } else if type_is_phantom_data(ty) {
        quote! {{
            <() as ::messagepack_core::decode::DecodeBorrowed<'__de>>::decode_borrowed_with_format(#format, __reader)?;
            ::core::marker::PhantomData
        }}
    } else {
        let replaced_ty = replace_lifetimes_in_type(ty, user_lifetimes);
        quote! {
            <#replaced_ty as ::messagepack_core::decode::DecodeBorrowed<'__de>>::decode_borrowed_with_format(#format, __reader)?
        }
    }
}
