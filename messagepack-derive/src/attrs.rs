use syn::{Attribute, Expr, ExprLit, Lit, Meta, Token, punctuated::Punctuated};

/// Container-level attributes (`#[msgpack(...)]` on the struct).
#[derive(Debug, Default)]
pub struct ContainerAttrs {
    /// Encode as a MessagePack map (default for named structs).
    pub map: bool,
    /// Encode as a MessagePack array.
    pub array: bool,
}

impl ContainerAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = ContainerAttrs::default();
        for attr in attrs {
            if !attr.path().is_ident("msgpack") {
                continue;
            }
            let nested = attr.parse_args_with(
                Punctuated::<Meta, Token![,]>::parse_terminated,
            )?;
            for meta in &nested {
                match meta {
                    Meta::Path(p) if p.is_ident("map") => {
                        result.map = true;
                    }
                    Meta::Path(p) if p.is_ident("array") => {
                        result.array = true;
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
        if result.map && result.array {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`map` and `array` are mutually exclusive",
            ));
        }
        Ok(result)
    }

    /// Whether the struct should be encoded as a map.
    /// Default is `true` for named-field structs, `false` for tuple structs.
    pub fn is_map(&self, is_named: bool) -> bool {
        if self.array {
            false
        } else if self.map {
            true
        } else {
            is_named
        }
    }
}

/// Field-level attributes (`#[msgpack(...)]` on a field).
#[derive(Debug, Default)]
pub struct FieldAttrs {
    /// Custom encode function path.
    pub encode_with: Option<syn::Path>,
    /// Custom decode function path.
    pub decode_with: Option<syn::Path>,
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
            let nested = attr.parse_args_with(
                Punctuated::<Meta, Token![,]>::parse_terminated,
            )?;
            for meta in &nested {
                match meta {
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
                        return Err(syn::Error::new_spanned(
                            other,
                            "unknown field attribute",
                        ));
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
