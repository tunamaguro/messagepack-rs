// Based on the approach used by serde and bitcode:
// https://github.com/serde-rs/serde/blob/master/serde_derive/src/bound.rs

use std::collections::HashSet;
use syn::punctuated::Pair;

/// Add `bound` to the where clause for each generic type parameter that
/// appears in at least one of `fields`. `PhantomData<T>` is hardcoded
/// to *not* require a bound so that a phantom type parameter compiles
/// even when it does not implement the trait.
pub fn with_bound(
    generics: &syn::Generics,
    fields: &[&syn::Field],
    bound: &syn::Path,
) -> syn::Generics {
    struct FindTyParams<'ast> {
        // Set of all generic type parameters on the current struct.
        all_type_params: HashSet<syn::Ident>,

        // Set of generic type parameters that appear in field types.
        relevant_type_params: HashSet<syn::Ident>,

        // Fields whose type is an associated type of a generic type parameter
        // (e.g. `T::Item`).
        associated_type_usage: Vec<&'ast syn::TypePath>,
    }

    impl<'ast> FindTyParams<'ast> {
        fn visit_field(&mut self, field: &'ast syn::Field) {
            if let syn::Type::Path(ty) = ungroup(&field.ty) {
                if let Some(Pair::Punctuated(t, _)) = ty.path.segments.pairs().next() {
                    if self.all_type_params.contains(&t.ident) {
                        self.associated_type_usage.push(ty);
                    }
                }
            }
            self.visit_type(&field.ty);
        }

        fn visit_path(&mut self, path: &'ast syn::Path) {
            if let Some(seg) = path.segments.last() {
                if seg.ident == "PhantomData" {
                    return;
                }
            }
            if path.leading_colon.is_none() && path.segments.len() == 1 {
                let id = &path.segments[0].ident;
                if self.all_type_params.contains(id) {
                    self.relevant_type_params.insert(id.clone());
                }
            }
            for segment in &path.segments {
                self.visit_path_segment(segment);
            }
        }

        fn visit_type(&mut self, ty: &'ast syn::Type) {
            match ty {
                syn::Type::Array(ty) => self.visit_type(&ty.elem),
                syn::Type::BareFn(ty) => {
                    for arg in &ty.inputs {
                        self.visit_type(&arg.ty);
                    }
                    self.visit_return_type(&ty.output);
                }
                syn::Type::Group(ty) => self.visit_type(&ty.elem),
                syn::Type::ImplTrait(ty) => {
                    for bound in &ty.bounds {
                        self.visit_type_param_bound(bound);
                    }
                }
                syn::Type::Macro(_) => {}
                syn::Type::Paren(ty) => self.visit_type(&ty.elem),
                syn::Type::Path(ty) => {
                    if let Some(qself) = &ty.qself {
                        self.visit_type(&qself.ty);
                    }
                    self.visit_path(&ty.path);
                }
                syn::Type::Ptr(ty) => self.visit_type(&ty.elem),
                syn::Type::Reference(ty) => self.visit_type(&ty.elem),
                syn::Type::Slice(ty) => self.visit_type(&ty.elem),
                syn::Type::TraitObject(ty) => {
                    for bound in &ty.bounds {
                        self.visit_type_param_bound(bound);
                    }
                }
                syn::Type::Tuple(ty) => {
                    for elem in &ty.elems {
                        self.visit_type(elem);
                    }
                }
                syn::Type::Infer(_) | syn::Type::Never(_) | syn::Type::Verbatim(_) => {}
                _ => {}
            }
        }

        fn visit_path_segment(&mut self, segment: &'ast syn::PathSegment) {
            self.visit_path_arguments(&segment.arguments);
        }

        fn visit_path_arguments(&mut self, arguments: &'ast syn::PathArguments) {
            match arguments {
                syn::PathArguments::None => {}
                syn::PathArguments::AngleBracketed(arguments) => {
                    for arg in &arguments.args {
                        match arg {
                            syn::GenericArgument::Type(arg) => self.visit_type(arg),
                            syn::GenericArgument::AssocType(arg) => self.visit_type(&arg.ty),
                            _ => {}
                        }
                    }
                }
                syn::PathArguments::Parenthesized(arguments) => {
                    for argument in &arguments.inputs {
                        self.visit_type(argument);
                    }
                    self.visit_return_type(&arguments.output);
                }
            }
        }

        fn visit_return_type(&mut self, return_type: &'ast syn::ReturnType) {
            match return_type {
                syn::ReturnType::Default => {}
                syn::ReturnType::Type(_, output) => self.visit_type(output),
            }
        }

        fn visit_type_param_bound(&mut self, bound: &'ast syn::TypeParamBound) {
            match bound {
                syn::TypeParamBound::Trait(bound) => self.visit_path(&bound.path),
                _ => {}
            }
        }
    }

    let all_type_params: HashSet<_> = generics
        .type_params()
        .map(|param| param.ident.clone())
        .collect();

    if all_type_params.is_empty() {
        return generics.clone();
    }

    let mut visitor = FindTyParams {
        all_type_params,
        relevant_type_params: HashSet::new(),
        associated_type_usage: Vec::new(),
    };

    for field in fields {
        visitor.visit_field(field);
    }

    let new_predicates = generics
        .type_params()
        .map(|param| param.ident.clone())
        .filter(|id| visitor.relevant_type_params.contains(id))
        .map(|id| syn::TypePath {
            qself: None,
            path: id.into(),
        })
        .chain(visitor.associated_type_usage.into_iter().cloned())
        .map(syn::Type::Path)
        .map(|bounded_ty| -> syn::WherePredicate {
            syn::parse_quote!(#bounded_ty: #bound)
        });

    let mut generics = generics.clone();
    generics
        .make_where_clause()
        .predicates
        .extend(new_predicates);
    generics
}

/// Add `bound` as a where-clause predicate for each concrete field type
/// in `types`. Unlike [`with_bound`], this does *not* inspect type
/// parameters — it bounds the type directly (e.g.
/// `Vec<u8>: AsRef<[u8]>`).
pub fn with_type_bound(
    generics: &syn::Generics,
    types: &[&syn::Type],
    bound: &syn::Path,
) -> syn::Generics {
    if types.is_empty() {
        return generics.clone();
    }
    let predicates = types.iter().map(|ty| -> syn::WherePredicate {
        syn::parse_quote!(#ty: #bound)
    });
    let mut generics = generics.clone();
    generics
        .make_where_clause()
        .predicates
        .extend(predicates);
    generics
}

/// Prepend the deserialization lifetime `de_lifetime` to the generic
/// parameters and add `de_lifetime: 'a` predicates for every existing
/// user lifetime `'a`.
pub fn with_de_lifetime(
    generics: &syn::Generics,
    de_lifetime: &syn::Lifetime,
) -> syn::Generics {
    let mut generics = generics.clone();

    generics.params.insert(
        0,
        syn::GenericParam::Lifetime(syn::LifetimeParam::new(de_lifetime.clone())),
    );

    let user_lifetimes: Vec<_> = generics
        .lifetimes()
        .filter(|lt| lt.lifetime != *de_lifetime)
        .map(|lt| lt.lifetime.clone())
        .collect();

    if !user_lifetimes.is_empty() {
        let where_clause = generics.make_where_clause();
        for user_lt in user_lifetimes {
            where_clause.predicates.push(syn::parse_quote! {
                #de_lifetime: #user_lt
            });
        }
    }

    generics
}

fn ungroup(mut ty: &syn::Type) -> &syn::Type {
    while let syn::Type::Group(group) = ty {
        ty = &group.elem;
    }
    ty
}
