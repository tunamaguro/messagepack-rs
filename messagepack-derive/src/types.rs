//! Shared type inspection, generic parameter analysis, and lifetime utilities
//! used by both the Encode and Decode derive implementations.

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;

// ---------------------------------------------------------------------------
// Type inspection helpers
// ---------------------------------------------------------------------------

/// Extract the inner type `T` from a wrapper like `Option<T>` or `Box<T>`.
pub fn single_type_argument<'a>(ty: &'a syn::Type, ident: &str) -> Option<&'a syn::Type> {
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

pub fn option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    single_type_argument(ty, "Option")
}

pub fn box_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    single_type_argument(ty, "Box")
}

pub fn type_is_phantom_data(ty: &syn::Type) -> bool {
    single_type_argument(ty, "PhantomData").is_some()
}

pub fn type_is_option(ty: &syn::Type) -> bool {
    option_inner_type(ty).is_some()
}

pub fn field_is_skipped_on_wire(field: &syn::Field) -> bool {
    type_is_phantom_data(&field.ty)
}

// ---------------------------------------------------------------------------
// Generic parameter dependency analysis
// ---------------------------------------------------------------------------

/// Collect types that transitively depend on the given set of type parameters.
///
/// For example, given `T` and a field of type `Vec<T>`, this pushes `Vec<T>`
/// (the outermost type that contains `T`) into `out`.
pub fn collect_dependent_types(
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

// ---------------------------------------------------------------------------
// Lifetime replacement
// ---------------------------------------------------------------------------

/// Replace every user-defined lifetime in `ty` with `'__de`.
pub fn replace_lifetimes_in_type(ty: &syn::Type, user_lifetimes: &HashSet<String>) -> TokenStream {
    let de_lifetime: syn::Lifetime = syn::parse_quote!('__de);
    replace_lifetimes_in_type_with(ty, user_lifetimes, &de_lifetime)
}

/// Replace every user-defined lifetime in `ty` with the given `replacement`.
pub fn replace_lifetimes_in_type_with(
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
