use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

use crate::attrs::{ContainerAttrs, FieldAttrs};

pub fn derive_encode(mut input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let (body, type_paths) = match &input.data {
        Data::Struct(data_struct) => {
            let type_paths = get_types_to_bound(&mut input.generics, &data_struct.fields);
            let body = encode_struct(name, &input.attrs, &data_struct.fields)?;
            (body, type_paths)
        }
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Encode derive is not yet supported for enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Encode derive is not supported for unions",
            ));
        }
    };

    {
        let where_clause = input.generics.make_where_clause();
        for type_path in type_paths {
            where_clause
                .predicates
                .push(syn::parse_quote!(#type_path: ::messagepack_core::encode::Encode));
        }
    }
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::messagepack_core::encode::Encode for #name #ty_generics
            #where_clause
        {
            fn encode<__W: ::messagepack_core::io::IoWrite>(&self, writer: &mut __W) -> ::core::result::Result<usize, ::messagepack_core::encode::Error<<__W as ::messagepack_core::io::IoWrite>::Error>> {
                #body
            }
        }
    })
}

fn get_types_to_bound(generics: &mut syn::Generics, fields: &Fields) -> Vec<syn::TypePath> {
    let type_param_idents: HashSet<String> = generics
        .type_params()
        .map(|param| param.ident.to_string())
        .collect();

    let mut dependent_types = Vec::new();
    for field in fields {
        let Ok(attrs) = FieldAttrs::from_attrs(&field.attrs) else {
            continue;
        };
        if attrs.encode_with.is_some() || attrs.bytes {
            continue;
        }

        collect_dependent_types(&field.ty, &type_param_idents, &mut dependent_types);
    }

    dependent_types
        .into_iter()
        .filter_map(|ty| {
            let syn::Type::Path(type_path) = ty else {
                return None;
            };
            Some(type_path)
        })
        .fold(Vec::new(), |mut out, type_path| {
            let type_tokens = quote!(#type_path).to_string();
            if out
                .iter()
                .any(|existing| quote!(#existing).to_string() == type_tokens)
            {
                return out;
            }
            out.push(type_path);
            out
        })
}

fn encode_struct(
    _name: &syn::Ident,
    attrs: &[syn::Attribute],
    fields: &Fields,
) -> syn::Result<TokenStream> {
    let container = ContainerAttrs::from_attrs(attrs)?;

    match fields {
        Fields::Named(named) => {
            let is_map = container.is_map(true);
            if is_map {
                encode_named_as_map(named)
            } else {
                encode_named_as_array(named)
            }
        }
        Fields::Unnamed(unnamed) => encode_tuple_struct(unnamed),
        Fields::Unit => {
            // Unit struct → encode as nil
            Ok(quote! {
                ::messagepack_core::encode::Encode::encode(&(), writer)
            })
        }
    }
}

fn encode_named_as_map(fields: &syn::FieldsNamed) -> syn::Result<TokenStream> {
    let num_fields = fields
        .named
        .iter()
        .filter(|field| !field_is_skipped_on_wire(field))
        .count();

    let mut field_encoders = Vec::new();
    for field in &fields.named {
        if field_is_skipped_on_wire(field) {
            continue;
        }

        let field_attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        let key_encode = quote! {
            __n += ::messagepack_core::encode::Encode::encode(&#field_name_str, writer)?;
        };

        let value_encode = encode_field_value(field, &field_attrs, quote! { &self.#field_name })?;

        field_encoders.push(quote! {
            #key_encode
            #value_encode
        });
    }

    Ok(quote! {
        let mut __n = 0usize;
        __n += ::messagepack_core::encode::Encode::encode(
            &::messagepack_core::encode::map::MapFormatEncoder::new(#num_fields),
            writer,
        )?;
        #(#field_encoders)*
        Ok(__n)
    })
}

fn encode_named_as_array(fields: &syn::FieldsNamed) -> syn::Result<TokenStream> {
    // Collect all fields with their key indices so skipped wire fields still
    // participate in contiguous key validation.
    let mut indexed_fields: Vec<(usize, &syn::Field)> = Vec::new();
    for field in &fields.named {
        let field_attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let key = field_attrs.key.ok_or_else(|| {
            syn::Error::new_spanned(
                field,
                "all fields must have `#[msgpack(key = N)]` when using `#[msgpack(array)]`",
            )
        })?;
        indexed_fields.push((key, field));
    }

    // Sort by key.
    indexed_fields.sort_by_key(|(k, _)| *k);

    // Validate keys are contiguous 0..N
    for (i, (k, field)) in indexed_fields.iter().enumerate() {
        if *k != i {
            return Err(syn::Error::new_spanned(
                field,
                format!(
                    "array keys must be contiguous starting from 0; expected key {}, found {}",
                    i, k
                ),
            ));
        }
    }

    let num_fields = indexed_fields
        .iter()
        .filter(|(_, field)| !field_is_skipped_on_wire(field))
        .count();

    let mut field_encoders = Vec::new();
    for (_key, field) in &indexed_fields {
        if field_is_skipped_on_wire(field) {
            continue;
        }

        let field_attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let field_name = field.ident.as_ref().unwrap();
        let value_encode = encode_field_value(field, &field_attrs, quote! { &self.#field_name })?;
        field_encoders.push(value_encode);
    }

    Ok(quote! {
        let mut __n = 0usize;
        __n += ::messagepack_core::encode::Encode::encode(
            &::messagepack_core::encode::array::ArrayFormatEncoder(#num_fields),
            writer,
        )?;
        #(#field_encoders)*
        Ok(__n)
    })
}

fn encode_tuple_struct(fields: &syn::FieldsUnnamed) -> syn::Result<TokenStream> {
    let num_fields = fields
        .unnamed
        .iter()
        .filter(|field| !field_is_skipped_on_wire(field))
        .count();

    let mut field_encoders = Vec::new();
    for (i, field) in fields.unnamed.iter().enumerate() {
        if field_is_skipped_on_wire(field) {
            continue;
        }

        let field_attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let idx = syn::Index::from(i);
        let value_encode = encode_field_value(field, &field_attrs, quote! { &self.#idx })?;
        field_encoders.push(value_encode);
    }

    Ok(quote! {
        let mut __n = 0usize;
        __n += ::messagepack_core::encode::Encode::encode(
            &::messagepack_core::encode::array::ArrayFormatEncoder(#num_fields),
            writer,
        )?;
        #(#field_encoders)*
        Ok(__n)
    })
}

fn encode_field_value(
    field: &syn::Field,
    attrs: &FieldAttrs,
    accessor: TokenStream,
) -> syn::Result<TokenStream> {
    if let Some(ref encode_fn) = attrs.encode_with {
        Ok(quote! {
            __n += #encode_fn(#accessor, writer)?;
        })
    } else if attrs.bytes {
        Ok(quote! {
            __n += ::messagepack_core::encode::Encode::encode(
                &::messagepack_core::encode::bin::BinaryEncoder(#accessor),
                writer,
            )?;
        })
    } else if let Some(inner_ty) = option_inner_type(&field.ty) {
        let some_encode = encode_type_value(inner_ty, quote! { __value });
        Ok(quote! {
            __n += match #accessor {
                ::core::option::Option::Some(__value) => #some_encode,
                ::core::option::Option::None => {
                    ::messagepack_core::encode::Encode::encode(&(), writer)?
                }
            };
        })
    } else {
        let value_encode = encode_type_value(&field.ty, accessor);
        Ok(quote! {
            __n += #value_encode;
        })
    }
}

fn encode_type_value(ty: &syn::Type, accessor: TokenStream) -> TokenStream {
    if let Some(inner_ty) = box_inner_type(ty) {
        let inner_encode = encode_type_value(inner_ty, quote! { &**#accessor });
        quote! { #inner_encode }
    } else if type_is_phantom_data(ty) {
        quote! {
            ::messagepack_core::encode::Encode::encode(&(), writer)?
        }
    } else {
        quote! {
            ::messagepack_core::encode::Encode::encode(#accessor, writer)?
        }
    }
}

fn option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    single_type_argument(ty, "Option")
}

fn box_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    single_type_argument(ty, "Box")
}

fn type_is_phantom_data(ty: &syn::Type) -> bool {
    single_type_argument(ty, "PhantomData").is_some()
}

fn field_is_skipped_on_wire(field: &syn::Field) -> bool {
    type_is_phantom_data(&field.ty)
}

fn single_type_argument<'a>(ty: &'a syn::Type, ident: &str) -> Option<&'a syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != ident {
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

fn collect_dependent_types(
    ty: &syn::Type,
    type_param_idents: &HashSet<String>,
    out: &mut Vec<syn::Type>,
) {
    if type_is_phantom_data(ty) {
        return;
    }

    match ty {
        syn::Type::Array(array) => collect_dependent_types(&array.elem, type_param_idents, out),
        syn::Type::Group(group) => collect_dependent_types(&group.elem, type_param_idents, out),
        syn::Type::Paren(paren) => collect_dependent_types(&paren.elem, type_param_idents, out),
        syn::Type::Ptr(ptr) => collect_dependent_types(&ptr.elem, type_param_idents, out),
        syn::Type::Reference(reference) => {
            collect_dependent_types(&reference.elem, type_param_idents, out)
        }
        syn::Type::Slice(slice) => collect_dependent_types(&slice.elem, type_param_idents, out),
        syn::Type::Tuple(tuple) => {
            for elem in &tuple.elems {
                collect_dependent_types(elem, type_param_idents, out);
            }
        }
        syn::Type::Path(type_path) => {
            if type_path_depends_on_params(type_path, type_param_idents) {
                push_unique_type(out, ty.clone());
                return;
            }

            for segment in &type_path.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        match arg {
                            syn::GenericArgument::Type(arg_ty) => {
                                collect_dependent_types(arg_ty, type_param_idents, out);
                            }
                            syn::GenericArgument::AssocType(assoc) => {
                                collect_dependent_types(&assoc.ty, type_param_idents, out);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn type_path_depends_on_params(
    type_path: &syn::TypePath,
    type_param_idents: &HashSet<String>,
) -> bool {
    if let Some(qself) = &type_path.qself {
        return type_depends_on_params(&qself.ty, type_param_idents);
    }

    type_path
        .path
        .segments
        .first()
        .map(|segment| type_param_idents.contains(&segment.ident.to_string()))
        .unwrap_or(false)
}

fn type_depends_on_params(ty: &syn::Type, type_param_idents: &HashSet<String>) -> bool {
    match ty {
        syn::Type::Path(type_path) => type_path_depends_on_params(type_path, type_param_idents),
        syn::Type::Reference(reference) => {
            type_depends_on_params(&reference.elem, type_param_idents)
        }
        syn::Type::Group(group) => type_depends_on_params(&group.elem, type_param_idents),
        syn::Type::Paren(paren) => type_depends_on_params(&paren.elem, type_param_idents),
        _ => false,
    }
}

fn push_unique_type(out: &mut Vec<syn::Type>, ty: syn::Type) {
    let ty_tokens = quote! { #ty }.to_string();
    if out
        .iter()
        .any(|existing| quote! { #existing }.to_string() == ty_tokens)
    {
        return;
    }
    out.push(ty);
}
