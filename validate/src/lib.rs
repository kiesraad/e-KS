use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Expr, Fields, LitStr, Path, Type, parse_macro_input};

/// Derive `Validate` implementations with field annotations.
///
/// Supported annotations:
/// - `#[validate(target = "Type", build = "path::to::builder")]` on the struct.
/// - `#[validate(with = "expr")]` on fields (repeatable to apply multiple validators in order),
///   where `expr` evaluates to `Fn(&str) -> Result<String, ValidationError>`.
/// - `#[validate(parse = "Type")]` to parse via `Type::from_str`.
/// - `#[validate(parse_with = "path", format = "...", ty = "Type")]` to parse via a custom function.
/// - `#[validate(optional)]` to treat empty strings as `None`.
/// - `#[validate(csrf)]` to validate CSRF tokens.
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_validate(&input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

struct StructOptions {
    target: Type,
    build: Path,
}

#[derive(Default)]
struct FieldOptions {
    optional: bool,
    csrf: bool,
    with_validators: Vec<Expr>,
    validator: Option<Validator>,
}

enum Validator {
    Parse {
        ty: Type,
    },
    ParseWith {
        path: Path,
        format: Option<Expr>,
        ty: Option<Type>,
    },
}

fn expand_validate(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_options = parse_struct_options(input)?;
    let struct_name = &input.ident;
    let validated_name = format_ident!("{}Validated", struct_name);
    let target = struct_options.target;
    let build_fn = struct_options.build;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields.named.iter().collect::<Vec<_>>(),
            _ => {
                return Err(syn::Error::new_spanned(
                    &data.fields,
                    "Validate can only be derived for structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Validate can only be derived for structs",
            ));
        }
    };

    let mut validated_fields = Vec::new();
    let mut field_inits = Vec::new();
    let mut field_blocks = Vec::new();

    for field in fields {
        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "expected named field"))?;
        let field_name = ident.to_string();
        let opts = parse_field_options(field)?;

        if opts.csrf {
            field_blocks.push(quote! {
                if !csrf_tokens.consume(&self.#ident) {
                    errors.push((
                        #field_name.to_string(),
                        crate::form::ValidationError::InvalidCsrfToken,
                    ));
                }
            });
            continue;
        }

        let (value_type, value_expr, validated) =
            build_field_validation(ident, &field_name, &field.ty, &opts)?;

        validated_fields.push(quote! {
            pub #ident: #value_type
        });
        if validated {
            field_blocks.push(build_field_block(ident, &value_expr));
            field_inits.push(quote! {
                #ident: #ident.expect("validated field")
            });
        } else {
            field_inits.push(quote! {
                #ident: #value_expr
            });
        }
    }

    let tokens = quote! {
        #[derive(Debug, Clone)]
        struct #validated_name {
            #(#validated_fields,)*
        }

        impl crate::form::Validate<#target> for #struct_name {
            fn validate(
                &self,
                current: Option<&#target>,
                csrf_tokens: &crate::form::CsrfTokens,
            ) -> Result<#target, crate::form::FormData<Self>> {
                let mut errors: crate::form::FieldErrors = Vec::new();

                #(#field_blocks)*

                if !errors.is_empty() {
                    tracing::debug!("Validation errors: {errors:?}");
                    return Err(crate::form::FormData::new_with_errors(
                        self.clone(),
                        csrf_tokens,
                        errors,
                    ));
                }

                let validated = #validated_name {
                    #(#field_inits,)*
                };

                Ok(#build_fn(validated, current))
            }
        }
    };

    Ok(tokens.into())
}

fn parse_struct_options(input: &DeriveInput) -> syn::Result<StructOptions> {
    let mut target = None;
    let mut build = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("validate") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("target") {
                let lit: LitStr = meta.value()?.parse()?;
                target = Some(lit.parse::<Type>()?);
                return Ok(());
            }
            if meta.path.is_ident("build") {
                let lit: LitStr = meta.value()?.parse()?;
                build = Some(lit.parse::<Path>()?);
                return Ok(());
            }

            Err(meta.error("unsupported validate attribute on struct"))
        })?;
    }

    let target = target.ok_or_else(|| {
        syn::Error::new_spanned(input, "missing #[validate(target = \"Type\")] on struct")
    })?;
    let build = build.ok_or_else(|| {
        syn::Error::new_spanned(input, "missing #[validate(build = \"path\")] on struct")
    })?;

    Ok(StructOptions { target, build })
}

fn parse_field_options(field: &syn::Field) -> syn::Result<FieldOptions> {
    let mut opts = FieldOptions::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("validate") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("optional") {
                opts.optional = true;
                return Ok(());
            }
            if meta.path.is_ident("csrf") {
                opts.csrf = true;
                return Ok(());
            }
            if meta.path.is_ident("with") {
                if opts.validator.is_some() {
                    return Err(meta.error("with cannot be combined with parse or parse_with"));
                }
                let lit: LitStr = meta.value()?.parse()?;
                let expr = lit.parse::<Expr>()?;
                opts.with_validators.push(expr);
                return Ok(());
            }
            if meta.path.is_ident("parse") {
                if opts.validator.is_some() || !opts.with_validators.is_empty() {
                    return Err(meta.error("only one validator kind is allowed per field"));
                }
                let lit: LitStr = meta.value()?.parse()?;
                let ty = lit.parse::<Type>()?;
                opts.validator = Some(Validator::Parse { ty });
                return Ok(());
            }
            if meta.path.is_ident("parse_with") {
                if opts.validator.is_some() || !opts.with_validators.is_empty() {
                    return Err(meta.error("only one validator kind is allowed per field"));
                }
                let lit: LitStr = meta.value()?.parse()?;
                let path = lit.parse::<Path>()?;
                opts.validator = Some(Validator::ParseWith {
                    path,
                    format: None,
                    ty: None,
                });
                return Ok(());
            }
            if meta.path.is_ident("format") {
                let expr: Expr = meta.value()?.parse()?;
                if let Some(Validator::ParseWith { format, .. }) = &mut opts.validator {
                    *format = Some(expr);
                    return Ok(());
                }
                return Err(meta.error("format requires parse_with"));
            }
            if meta.path.is_ident("ty") {
                let lit: LitStr = meta.value()?.parse()?;
                let ty = lit.parse::<Type>()?;
                if let Some(Validator::ParseWith { ty: stored, .. }) = &mut opts.validator {
                    *stored = Some(ty);
                    return Ok(());
                }
                return Err(meta.error("ty requires parse_with"));
            }

            Err(meta.error("unsupported validate attribute on field"))
        })?;
    }

    if let Some(Validator::ParseWith { ty, .. }) = &opts.validator
        && ty.is_none()
    {
        return Err(syn::Error::new_spanned(
            field,
            "parse_with requires ty = \"Type\"",
        ));
    }

    Ok(opts)
}

fn build_field_validation(
    ident: &syn::Ident,
    field_name: &str,
    field_ty: &Type,
    opts: &FieldOptions,
) -> syn::Result<(Type, proc_macro2::TokenStream, bool)> {
    let base_ty = if !opts.with_validators.is_empty() {
        syn::parse_quote!(String)
    } else {
        match &opts.validator {
            Some(Validator::Parse { ty }) => ty.clone(),
            Some(Validator::ParseWith { ty, .. }) => ty
                .clone()
                .ok_or_else(|| syn::Error::new_spanned(field_ty, "missing parse_with ty"))?,
            None => return Ok((field_ty.clone(), quote!(self.#ident.clone()), false)),
        }
    };

    let output_ty = if opts.optional {
        syn::parse_quote!(Option<#base_ty>)
    } else {
        base_ty
    };

    let expr = if !opts.with_validators.is_empty() {
        let validators = opts.with_validators.iter().map(|expr| {
            quote! {
                if ok {
                    let validator = #expr;
                    match validator(&value) {
                        Ok(next_value) => value = next_value,
                        Err(err) => {
                            errors.push((#field_name.to_string(), err));
                            ok = false;
                        }
                    }
                }
            }
        });

        if opts.optional {
            quote!({
                if self.#ident.is_empty() {
                    Some(None)
                } else {
                    let mut value = self.#ident.trim().to_string();
                    let mut ok = true;
                    #(#validators)*
                    if ok { Some(Some(value)) } else { None }
                }
            })
        } else {
            quote!({
                let mut value = self.#ident.clone();
                let mut ok = true;
                #(#validators)*
                if ok { Some(value) } else { None }
            })
        }
    } else {
        match &opts.validator {
            Some(Validator::Parse { ty }) => {
                if opts.optional {
                    quote!({
                        let value = self.#ident.trim();
                        if self.#ident.is_empty() {
                            Some(None)
                        } else {
                            match #ty::from_str(value) {
                                Ok(value) => Some(Some(value)),
                                Err(_) => {
                                    errors.push((
                                        #field_name.to_string(),
                                        crate::form::ValidationError::InvalidValue,
                                    ));
                                    None
                                }
                            }
                        }
                    })
                } else {
                    quote!({
                        let value = self.#ident.trim();
                        if value.is_empty() {
                            errors.push((
                                #field_name.to_string(),
                                crate::form::ValidationError::ValueShouldNotBeEmpty,
                            ));
                            None
                        } else {
                            match #ty::from_str(value) {
                                Ok(value) => Some(value),
                                Err(_) => {
                                    errors.push((
                                        #field_name.to_string(),
                                        crate::form::ValidationError::InvalidValue,
                                    ));
                                    None
                                }
                            }
                        }
                    })
                }
            }
            Some(Validator::ParseWith { path, format, .. }) => {
                let call = if let Some(format) = format {
                    quote!(#path(value, #format))
                } else {
                    quote!(#path(value))
                };
                if opts.optional {
                    quote!({
                        let value = self.#ident.trim();
                        if self.#ident.is_empty() {
                            Some(None)
                        } else {
                            match #call {
                                Ok(value) => Some(Some(value)),
                                Err(_) => {
                                    errors.push((
                                        #field_name.to_string(),
                                        crate::form::ValidationError::InvalidValue,
                                    ));
                                    None
                                }
                            }
                        }
                    })
                } else {
                    quote!({
                        let value = self.#ident.trim();
                        if value.is_empty() {
                            errors.push((
                                #field_name.to_string(),
                                crate::form::ValidationError::ValueShouldNotBeEmpty,
                            ));
                            None
                        } else {
                            match #call {
                                Ok(value) => Some(value),
                                Err(_) => {
                                    errors.push((
                                        #field_name.to_string(),
                                        crate::form::ValidationError::InvalidValue,
                                    ));
                                    None
                                }
                            }
                        }
                    })
                }
            }
            None => quote!(self.#ident.clone()),
        }
    };

    Ok((output_ty, expr, true))
}

fn build_field_block(
    ident: &syn::Ident,
    value_expr: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        let #ident = #value_expr;
    }
}
