use syn::{Attribute, Expr, ExprLit, Lit, Meta, Token, punctuated::Punctuated};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatKind {
    Map,
    Array,
}

/// Container-level attributes (`#[msgpack(...)]` on the struct).
#[derive(Debug)]
pub struct ContainerAttrs {
    pub format_kind: FormatKind,
    pub bounds: Option<proc_macro2::TokenStream>,
}

impl ContainerAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut map_ident = false;
        let mut array_ident = false;
        let mut bounds = None;
        for attr in attrs {
            if !attr.path().is_ident("msgpack") {
                continue;
            }
            let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            for meta in &nested {
                match meta {
                    Meta::Path(p) if p.is_ident("map") => {
                        if map_ident {
                            return Err(syn::Error::new_spanned(p, "duplicate `map` attribute"));
                        }
                        map_ident = true;
                    }
                    Meta::Path(p) if p.is_ident("array") => {
                        if array_ident {
                            return Err(syn::Error::new_spanned(p, "duplicate `array` attribute"));
                        }
                        array_ident = true;
                    }
                    Meta::NameValue(nv) if nv.path.is_ident("bound") => {
                        if bounds.is_some() {
                            return Err(syn::Error::new_spanned(nv, "duplicate `bound` attribute"));
                        }
                        let bound_str = match &nv.value {
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(s), ..
                            }) => s.value(),
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    &nv.value,
                                    "expected a string literal",
                                ));
                            }
                        };
                        bounds = Some(bound_str.parse()?);
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            other,
                            "unknown container attribute",
                        ));
                    }
                }
            }
        }
        if map_ident && array_ident {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`map` and `array` are mutually exclusive",
            ));
        }
        Ok(Self {
            format_kind: if array_ident {
                FormatKind::Array
            } else {
                FormatKind::Map
            },
            bounds,
        })
    }
}

/// Field-level attributes (`#[msgpack(...)]` on a field).
#[derive(Debug, Default)]
pub struct FieldAttrs {
    /// Custom encode function path.
    pub encode_with: Option<syn::Path>,
    /// Custom decode function path.
    pub decode_with: Option<syn::Path>,
    /// Use `Default::default()` when the field is missing during decode.
    pub default: bool,
    /// Encode/decode as binary.
    pub bytes: bool,
    /// Array key index (required for `#[msgpack(array)]` mode).
    pub key: Option<usize>,
}

impl FieldAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = FieldAttrs::default();
        for attr in attrs {
            if !attr.path().is_ident("msgpack") {
                continue;
            }
            let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            for meta in &nested {
                match meta {
                    Meta::Path(p) if p.is_ident("default") => {
                        result.default = true;
                    }
                    Meta::Path(p) if p.is_ident("bytes") => {
                        result.bytes = true;
                    }
                    Meta::NameValue(nv) if nv.path.is_ident("encode_with") => {
                        let path = parse_lit_str_as_path(&nv.value)?;
                        result.encode_with = Some(path);
                    }
                    Meta::NameValue(nv) if nv.path.is_ident("decode_with") => {
                        let path = parse_lit_str_as_path(&nv.value)?;
                        result.decode_with = Some(path);
                    }
                    Meta::NameValue(nv) if nv.path.is_ident("key") => {
                        let idx = parse_usize(&nv.value)?;
                        result.key = Some(idx);
                    }
                    other => {
                        return Err(syn::Error::new_spanned(other, "unknown field attribute"));
                    }
                }
            }
        }
        Ok(result)
    }
}

fn parse_lit_str_as_path(expr: &Expr) -> syn::Result<syn::Path> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => s.parse(),
        _ => Err(syn::Error::new_spanned(expr, "expected a string literal")),
    }
}

fn parse_usize(expr: &Expr) -> syn::Result<usize> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit), ..
        }) => lit.base10_parse::<usize>(),
        _ => Err(syn::Error::new_spanned(expr, "expected an integer literal")),
    }
}
