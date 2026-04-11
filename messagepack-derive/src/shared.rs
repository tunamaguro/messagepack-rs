use std::collections::HashSet;

use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Error, Field, Fields, GenericArgument, Generics, Ident, LitInt,
    LitStr, Member, Meta, MetaList, Path, PathArguments, Type, TypeArray, TypeGroup, TypeParen,
    TypePath, TypeReference, TypeSlice, TypeTuple, WhereClause, parse_quote,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeriveKind {
    Encode,
    Decode,
}

impl DeriveKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Encode => "Encode",
            Self::Decode => "Decode",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerMode {
    Map,
    Array,
}

#[derive(Debug, Clone, Default)]
pub struct ContainerAttrs {
    pub mode: Option<ContainerMode>,
}

#[derive(Debug, Clone, Default)]
pub struct FieldAttrs {
    pub key: Option<usize>,
    pub bytes: bool,
    pub default: bool,
    pub encode_with: Option<Path>,
    pub decode_with: Option<Path>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub member: Member,
    pub ty: Type,
    pub attrs: FieldAttrs,
    pub span: Span,
    pub name: Option<Ident>,
    pub key_name: Option<String>,
    pub is_phantom: bool,
}

impl FieldInfo {
    pub fn is_skipped_for_encode(&self) -> bool {
        self.is_phantom
    }

    pub fn is_skipped_for_decode(&self) -> bool {
        self.is_phantom
    }
}

#[derive(Debug, Clone)]
pub enum StructStyle {
    Named(Vec<FieldInfo>),
    Tuple(Vec<FieldInfo>),
    Unit,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub ident: Ident,
    pub generics: Generics,
    pub container: ContainerAttrs,
    pub style: StructStyle,
}

pub fn parse_struct(input: DeriveInput, kind: DeriveKind) -> syn::Result<StructInfo> {
    let ident = input.ident;
    let generics = input.generics;
    let container = parse_container_attrs(&input.attrs)?;

    let style = match input.data {
        Data::Struct(data) => parse_fields(data.fields)?,
        Data::Enum(data) => {
            return Err(Error::new(
                data.enum_token.span,
                format!("{} derive is not yet supported for enums", kind.name()),
            ));
        }
        Data::Union(data) => {
            return Err(Error::new(
                data.union_token.span,
                format!("{} derive is only supported for structs", kind.name()),
            ));
        }
    };

    Ok(StructInfo {
        ident,
        generics,
        container,
        style,
    })
}

fn parse_fields(fields: Fields) -> syn::Result<StructStyle> {
    match fields {
        Fields::Named(named) => Ok(StructStyle::Named(
            named
                .named
                .into_iter()
                .map(parse_named_field)
                .collect::<syn::Result<Vec<_>>>()?,
        )),
        Fields::Unnamed(unnamed) => Ok(StructStyle::Tuple(
            unnamed
                .unnamed
                .into_iter()
                .enumerate()
                .map(|(index, field)| parse_unnamed_field(index, field))
                .collect::<syn::Result<Vec<_>>>()?,
        )),
        Fields::Unit => Ok(StructStyle::Unit),
    }
}

fn parse_named_field(field: Field) -> syn::Result<FieldInfo> {
    let attrs = parse_field_attrs(&field.attrs)?;
    let ident = field
        .ident
        .clone()
        .ok_or_else(|| Error::new(field.span(), "expected named field"))?;
    Ok(FieldInfo {
        member: Member::Named(ident.clone()),
        ty: field.ty.clone(),
        attrs,
        span: field.span(),
        name: Some(ident.clone()),
        key_name: Some(ident.to_string()),
        is_phantom: is_phantom_data(&field.ty),
    })
}

fn parse_unnamed_field(index: usize, field: Field) -> syn::Result<FieldInfo> {
    let attrs = parse_field_attrs(&field.attrs)?;
    Ok(FieldInfo {
        member: Member::Unnamed(index.into()),
        ty: field.ty.clone(),
        attrs,
        span: field.span(),
        name: None,
        key_name: None,
        is_phantom: is_phantom_data(&field.ty),
    })
}

fn parse_container_attrs(attrs: &[Attribute]) -> syn::Result<ContainerAttrs> {
    let mut out = ContainerAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("msgpack") {
            continue;
        }
        match &attr.meta {
            Meta::List(list) => parse_container_list(list, &mut out)?,
            other => {
                return Err(Error::new(
                    other.span(),
                    "expected #[msgpack(...)] container attribute",
                ));
            }
        }
    }

    Ok(out)
}

fn parse_container_list(list: &MetaList, out: &mut ContainerAttrs) -> syn::Result<()> {
    list.parse_nested_meta(|meta| {
        if meta.path.is_ident("map") {
            if out.mode.is_some() {
                return Err(meta.error("`map` and `array` are mutually exclusive"));
            }
            out.mode = Some(ContainerMode::Map);
            return Ok(());
        }
        if meta.path.is_ident("array") {
            if out.mode.is_some() {
                return Err(meta.error("`map` and `array` are mutually exclusive"));
            }
            out.mode = Some(ContainerMode::Array);
            return Ok(());
        }

        Err(meta.error("unsupported container attribute"))
    })
}

fn parse_field_attrs(attrs: &[Attribute]) -> syn::Result<FieldAttrs> {
    let mut out = FieldAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("msgpack") {
            continue;
        }
        match &attr.meta {
            Meta::List(list) => {
                list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("key") {
                        let value = meta.value()?;
                        let lit: LitInt = value.parse()?;
                        out.key = Some(lit.base10_parse()?);
                        return Ok(());
                    }
                    if meta.path.is_ident("bytes") {
                        out.bytes = true;
                        return Ok(());
                    }
                    if meta.path.is_ident("default") {
                        out.default = true;
                        return Ok(());
                    }
                    if meta.path.is_ident("encode_with") {
                        let value = meta.value()?;
                        let lit: LitStr = value.parse()?;
                        out.encode_with = Some(lit.parse()?);
                        return Ok(());
                    }
                    if meta.path.is_ident("decode_with") {
                        let value = meta.value()?;
                        let lit: LitStr = value.parse()?;
                        out.decode_with = Some(lit.parse()?);
                        return Ok(());
                    }
                    Err(meta.error("unsupported field attribute"))
                })?;
            }
            other => {
                return Err(Error::new(
                    other.span(),
                    "expected #[msgpack(...)] field attribute",
                ));
            }
        }
    }

    Ok(out)
}

pub fn is_phantom_data(ty: &Type) -> bool {
    match strip_groups(ty) {
        Type::Path(TypePath { qself: None, path }) => path
            .segments
            .last()
            .map(|seg| seg.ident == "PhantomData")
            .unwrap_or(false),
        _ => false,
    }
}

pub fn box_inner(ty: &Type) -> Option<Type> {
    let Type::Path(TypePath { qself: None, path }) = strip_groups(ty) else {
        return None;
    };
    let seg = path.segments.last()?;
    if seg.ident != "Box" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    match args.args.first() {
        Some(GenericArgument::Type(inner)) => Some(inner.clone()),
        _ => None,
    }
}

pub fn option_inner(ty: &Type) -> Option<Type> {
    let Type::Path(TypePath { qself: None, path }) = strip_groups(ty) else {
        return None;
    };
    let seg = path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    match args.args.first() {
        Some(GenericArgument::Type(inner)) => Some(inner.clone()),
        _ => None,
    }
}

pub fn replace_lifetimes(ty: &Type, replacement: &syn::Lifetime) -> Type {
    match ty {
        Type::Reference(TypeReference {
            and_token,
            lifetime,
            mutability,
            elem,
        }) => Type::Reference(TypeReference {
            and_token: *and_token,
            lifetime: lifetime.as_ref().map(|lt| {
                if lt.ident == "static" {
                    lt.clone()
                } else {
                    replacement.clone()
                }
            }),
            mutability: *mutability,
            elem: Box::new(replace_lifetimes(elem, replacement)),
        }),
        Type::Path(TypePath { qself, path }) => {
            let mut new = path.clone();
            for seg in &mut new.segments {
                if let PathArguments::AngleBracketed(args) = &mut seg.arguments {
                    for arg in &mut args.args {
                        match arg {
                            GenericArgument::Lifetime(lt) => {
                                if lt.ident != "static" {
                                    *lt = replacement.clone();
                                }
                            }
                            GenericArgument::Type(inner) => {
                                *inner = replace_lifetimes(inner, replacement);
                            }
                            GenericArgument::AssocType(binding) => {
                                binding.ty = replace_lifetimes(&binding.ty, replacement);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Type::Path(TypePath {
                qself: qself.clone(),
                path: new,
            })
        }
        Type::Tuple(TypeTuple { paren_token, elems }) => Type::Tuple(TypeTuple {
            paren_token: *paren_token,
            elems: elems
                .iter()
                .map(|elem| replace_lifetimes(elem, replacement))
                .collect(),
        }),
        Type::Slice(TypeSlice {
            bracket_token,
            elem,
        }) => Type::Slice(TypeSlice {
            bracket_token: *bracket_token,
            elem: Box::new(replace_lifetimes(elem, replacement)),
        }),
        Type::Array(TypeArray {
            bracket_token,
            elem,
            semi_token,
            len,
        }) => Type::Array(TypeArray {
            bracket_token: *bracket_token,
            elem: Box::new(replace_lifetimes(elem, replacement)),
            semi_token: *semi_token,
            len: len.clone(),
        }),
        Type::Group(TypeGroup { group_token, elem }) => Type::Group(TypeGroup {
            group_token: *group_token,
            elem: Box::new(replace_lifetimes(elem, replacement)),
        }),
        Type::Paren(TypeParen { paren_token, elem }) => Type::Paren(TypeParen {
            paren_token: *paren_token,
            elem: Box::new(replace_lifetimes(elem, replacement)),
        }),
        other => other.clone(),
    }
}

pub fn add_type_bound(generics: &mut Generics, ty: Type, bound: syn::TypeParamBound) {
    generics
        .make_where_clause()
        .predicates
        .push(syn::WherePredicate::Type(syn::PredicateType {
            lifetimes: None,
            bounded_ty: ty,
            colon_token: Default::default(),
            bounds: [bound].into_iter().collect(),
        }));
}

pub fn ensure_where_clause(generics: &mut Generics) -> &mut WhereClause {
    generics.make_where_clause()
}

pub fn collect_bound_types(ty: &Type, generics: &Generics) -> Vec<Type> {
    let type_params = generics
        .type_params()
        .map(|tp| tp.ident.to_string())
        .collect::<HashSet<_>>();
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    collect_bound_types_inner(ty, &type_params, &mut seen, &mut out);
    out
}

fn collect_bound_types_inner(
    ty: &Type,
    type_params: &HashSet<String>,
    seen: &mut HashSet<String>,
    out: &mut Vec<Type>,
) {
    match strip_groups(ty) {
        Type::Reference(TypeReference { elem, .. }) => {
            collect_bound_types_inner(elem, type_params, seen, out);
        }
        Type::Slice(TypeSlice { elem, .. }) => {
            collect_bound_types_inner(elem, type_params, seen, out);
        }
        Type::Array(TypeArray { elem, .. }) => {
            collect_bound_types_inner(elem, type_params, seen, out);
        }
        Type::Tuple(TypeTuple { elems, .. }) => {
            for elem in elems {
                collect_bound_types_inner(elem, type_params, seen, out);
            }
        }
        Type::Path(TypePath { qself, path }) => {
            if let Some(qself) = qself {
                collect_bound_types_inner(&qself.ty, type_params, seen, out);
            }

            let first_is_type_param = path
                .segments
                .first()
                .map(|segment| type_params.contains(&segment.ident.to_string()))
                .unwrap_or(false);
            if first_is_type_param {
                let key = ty.to_token_stream().to_string();
                if seen.insert(key) {
                    out.push(ty.clone());
                }
                return;
            }

            for segment in &path.segments {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        match arg {
                            GenericArgument::Type(inner) => {
                                collect_bound_types_inner(inner, type_params, seen, out);
                            }
                            GenericArgument::AssocType(binding) => {
                                collect_bound_types_inner(&binding.ty, type_params, seen, out);
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

fn strip_groups(mut ty: &Type) -> &Type {
    loop {
        match ty {
            Type::Group(group) => ty = &group.elem,
            Type::Paren(paren) => ty = &paren.elem,
            _ => return ty,
        }
    }
}

pub fn decode_lifetime() -> syn::Lifetime {
    parse_quote!('__msgpack_de)
}
