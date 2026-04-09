use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, WherePredicate};

use crate::attrs::{ContainerAttrs, FieldAttrs};

pub fn derive_decode(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (_, _ty_generics, where_clause) = input.generics.split_for_impl();

    // Collect user-defined lifetime names so we can replace them with '__de.
    let user_lifetimes: HashSet<String> = input
        .generics
        .lifetimes()
        .map(|lt| lt.lifetime.ident.to_string())
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
    for predicate in decode_field_bounds(&input.data, &user_lifetimes)? {
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

fn decode_field_bounds(
    data: &Data,
    user_lifetimes: &HashSet<String>,
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
        } else if let Some(inner_ty) = option_inner_type(&field.ty) {
            let inner_decode_ty = replace_lifetimes_in_type(inner_ty, user_lifetimes);
            predicates.push(syn::parse_quote!(
                #inner_decode_ty: ::messagepack_core::decode::DecodeBorrowed<'__de, Value = #inner_decode_ty>
            ));
        } else {
            predicates.push(syn::parse_quote!(
                #decode_ty: ::messagepack_core::decode::DecodeBorrowed<'__de, Value = #decode_ty>
            ));
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
    let num_fields = fields.named.len();

    // Collect field info
    struct FieldInfo<'a> {
        ident: &'a syn::Ident,
        name_str: String,
        attrs: FieldAttrs,
        ty: &'a syn::Type,
        array_key: usize,
        allow_missing: bool,
    }

    let mut field_infos: Vec<FieldInfo> = Vec::new();
    for (i, field) in fields.named.iter().enumerate() {
        let ident = field.ident.as_ref().unwrap();
        let attrs = FieldAttrs::from_attrs(&field.attrs)?;
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
            allow_missing: attrs.default || type_is_option(&field.ty),
            attrs,
            ty: &field.ty,
            array_key,
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
        .map(|fi| {
            let var = format_ident!("__field_{}", fi.ident);
            let decode_expr = decode_field_expr(fi.ty, &fi.attrs, user_lifetimes);
            quote! {
                #var = ::core::option::Option::Some(#decode_expr);
            }
        })
        .collect();

    let missing_allowed_count = field_infos.iter().filter(|fi| fi.allow_missing).count();

    // Generate constructor: unwrap each Option.
    let constructor_fields: Vec<TokenStream> = field_infos
        .iter()
        .map(|fi| {
            let ident = fi.ident;
            let var = format_ident!("__field_{}", fi.ident);
            if fi.allow_missing {
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

                let __required_fields = #num_fields - #missing_allowed_count;
                if __len < __required_fields || __len > #num_fields {
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
    let num_fields = fields.unnamed.len();

    let mut decode_stmts = Vec::new();
    let mut field_vars = Vec::new();
    for (i, field) in fields.unnamed.iter().enumerate() {
        let field_attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let var = format_ident!("__field_{}", i);
        let decode_expr = decode_field_expr(&field.ty, &field_attrs, user_lifetimes);
        let output_ty = replace_lifetimes_in_type(&field.ty, user_lifetimes);
        decode_stmts.push(quote! {
            let #var: #output_ty = #decode_expr;
        });
        field_vars.push(var);
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
    } else if let Some(inner_ty) = option_inner_type(ty) {
        let replaced_inner_ty = replace_lifetimes_in_type(inner_ty, user_lifetimes);
        quote! {{
            let __field_format =
                <::messagepack_core::Format as ::messagepack_core::decode::DecodeBorrowed<'__de>>::decode_borrowed(__reader)?;
            match __field_format {
                ::messagepack_core::Format::Nil => ::core::option::Option::None,
                __other => ::core::option::Option::Some(
                    <#replaced_inner_ty as ::messagepack_core::decode::DecodeBorrowed<'__de>>
                        ::decode_borrowed_with_format(__other, __reader)?
                ),
            }
        }}
    } else {
        let replaced_ty = replace_lifetimes_in_type(ty, user_lifetimes);
        quote! {
            <#replaced_ty as ::messagepack_core::decode::DecodeBorrowed<'__de>>::decode_borrowed(__reader)?
        }
    }
}

fn type_is_option(ty: &syn::Type) -> bool {
    option_inner_type(ty).is_some()
}

fn option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    let first = args.args.first()?;
    let syn::GenericArgument::Type(inner_ty) = first else {
        return None;
    };

    Some(inner_ty)
}

fn replace_lifetimes_in_type_with(
    ty: &syn::Type,
    user_lifetimes: &HashSet<String>,
    replacement: &syn::Lifetime,
) -> TokenStream {
    if user_lifetimes.is_empty() {
        return quote! { #ty };
    }
    let tokens = quote! { #ty };
    replace_lifetimes_in_tokens(tokens, user_lifetimes, replacement)
}

/// Replace user-defined lifetimes with `'__de` in a type's token stream.
fn replace_lifetimes_in_type(ty: &syn::Type, user_lifetimes: &HashSet<String>) -> TokenStream {
    let de_lifetime: syn::Lifetime = syn::parse_quote!('__de);
    replace_lifetimes_in_type_with(ty, user_lifetimes, &de_lifetime)
}

fn replace_lifetimes_in_tokens(
    tokens: TokenStream,
    user_lifetimes: &HashSet<String>,
    replacement: &syn::Lifetime,
) -> TokenStream {
    use proc_macro2::TokenTree;
    let mut result = TokenStream::new();
    let mut iter = tokens.into_iter().peekable();
    while let Some(tt) = iter.next() {
        match &tt {
            TokenTree::Punct(p) if p.as_char() == '\'' => {
                // Check if next token is one of the user lifetimes
                if let Some(TokenTree::Ident(ident)) = iter.peek()
                    && user_lifetimes.contains(&ident.to_string())
                {
                    result.extend(quote! { #replacement });
                    iter.next(); // consume the ident
                    continue;
                }
                result.extend(core::iter::once(tt));
            }
            TokenTree::Group(g) => {
                let replaced = replace_lifetimes_in_tokens(g.stream(), user_lifetimes, replacement);
                let new_group = proc_macro2::Group::new(g.delimiter(), replaced);
                result.extend(core::iter::once(TokenTree::Group(new_group)));
            }
            _ => result.extend(core::iter::once(tt)),
        }
    }
    result
}
