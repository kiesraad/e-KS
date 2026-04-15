use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, Type, parse_macro_input};

/// Derive `Validate` implementations with field annotations.
///
/// Supported annotations:
/// - `#[validate(target = "Type")]` on the struct.
/// - `#[validate(parse = "Type")]` to parse via `Type::from_str`.
/// - `#[validate(optional)]` to treat empty strings as `None`.
/// - `#[validate(not_empty)]` to reject empty values via `is_empty`.
/// - `#[validate(csrf)]` to validate CSRF tokens.
/// - `#[validate(ignore)]` to skip validation and mapping for a field.
/// - `#[validate(flatten)]` to validate a nested form and prefix its errors with `field.child`.
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_validate(&input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

#[derive(Default)]
struct FieldOptions {
    optional: bool,
    csrf: bool,
    ignore: bool,
    parse_ty: Option<Type>,
    flatten: bool,
    not_empty: bool,
}

/// Expand the `Validate` derive into an implementation for the target type.
///
/// Example:
/// - `PersonForm` + `#[validate(target = "Person")]` -> `impl Validate<Person> for PersonForm`.
fn expand_validate(input: &DeriveInput) -> syn::Result<TokenStream> {
    let target = parse_struct_options(input)?;
    let struct_name = &input.ident;

    let fields = collect_named_fields(input)?;
    let field_blocks = build_field_blocks(&fields)?;
    let tokens = build_validate_impl(struct_name, &target, &field_blocks);

    Ok(tokens.into())
}

struct FieldBlocks {
    field_inits: Vec<proc_macro2::TokenStream>,
    field_blocks_create: Vec<proc_macro2::TokenStream>,
    field_blocks_update: Vec<proc_macro2::TokenStream>,
    has_csrf: bool,
}

fn collect_named_fields(input: &DeriveInput) -> syn::Result<Vec<&syn::Field>> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(fields.named.iter().collect()),
            _ => Err(syn::Error::new_spanned(
                &data.fields,
                "Validate can only be derived for structs with named fields",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            input,
            "Validate can only be derived for structs",
        )),
    }
}

fn build_field_blocks(fields: &[&syn::Field]) -> syn::Result<FieldBlocks> {
    let mut field_inits = Vec::new();
    let mut field_blocks_create = Vec::new();
    let mut field_blocks_update = Vec::new();
    let mut has_csrf = false;

    for field in fields {
        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "expected named field"))?;
        let field_name = ident.to_string();
        let opts = parse_field_options(field)?;

        if opts.csrf {
            has_csrf = true;
            let block = quote! {
                if !csrf_tokens.consume(&self.#ident) {
                    errors.push((
                        #field_name.to_string(),
                        crate::form::ValidationError::InvalidCsrfToken,
                    ));
                }
            };
            field_blocks_create.push(block.clone());
            field_blocks_update.push(block);
            continue;
        }
        if opts.ignore {
            continue;
        }

        if opts.not_empty {
            let error = if is_type_named(&field.ty, "Vec") {
                quote!(crate::form::ValidationError::ChooseAtLeastOneOption)
            } else {
                quote!(crate::form::ValidationError::ValueShouldNotBeEmpty)
            };
            let block = quote! {
                if self.#ident.is_empty() {
                    errors.push((#field_name.to_string(), #error));
                }
            };
            field_blocks_create.push(block.clone());
            field_blocks_update.push(block);
        }

        let validation = build_field_validation(ident, &field_name, &field.ty, &opts)?;

        field_blocks_create.push(build_field_block(ident, &validation.create_expr));
        field_blocks_update.push(build_field_block(ident, &validation.update_expr));
        if validation.validated {
            field_inits.push(quote! {
                #ident: #ident.expect("validated field")
            });
        } else {
            field_inits.push(quote! {
                #ident: #ident
            });
        }
    }

    Ok(FieldBlocks {
        field_inits,
        field_blocks_create,
        field_blocks_update,
        has_csrf,
    })
}

fn is_type_named(ty: &Type, name: &str) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    type_path
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

fn build_with_csrf_impl(struct_name: &syn::Ident, has_csrf: bool) -> proc_macro2::TokenStream {
    if has_csrf {
        quote! {
            impl crate::form::WithCsrfToken for #struct_name {
                fn with_csrf_token(self, csrf_token: crate::form::CsrfToken) -> Self {
                    #[allow(clippy::needless_update)]
                    #struct_name {
                        csrf_token: csrf_token.value,
                        ..self
                    }
                }
            }
        }
    } else {
        quote! {
            impl crate::form::WithCsrfToken for #struct_name {
                fn with_csrf_token(self, _csrf_token: crate::form::CsrfToken) -> Self {
                    self
                }
            }
        }
    }
}

fn build_validate_impl(
    struct_name: &syn::Ident,
    target: &Type,
    field_blocks: &FieldBlocks,
) -> proc_macro2::TokenStream {
    let with_csrf_impl = build_with_csrf_impl(struct_name, field_blocks.has_csrf);
    let FieldBlocks {
        field_inits,
        field_blocks_create,
        field_blocks_update,
        ..
    } = field_blocks;

    quote! {
        #with_csrf_impl

        impl #struct_name {
            pub fn validate_create(
                self,
                csrf_tokens: &crate::form::CsrfTokens,
            ) -> Result<#target, crate::form::FormData<Self>> {
                let mut errors: crate::form::FieldErrors = Vec::new();

                #(#field_blocks_create)*

                if !errors.is_empty() {
                    tracing::debug!("Validation errors: {errors:?}");
                    return Err(crate::form::FormData::new_with_errors(
                        self,
                        csrf_tokens,
                        errors,
                    ));
                }

                #[allow(clippy::needless_update)]
                Ok(#target {
                    #(#field_inits,)*
                    ..Default::default()
                })
            }

            pub fn validate_update(
                self,
                current: &#target,
                csrf_tokens: &crate::form::CsrfTokens,
            ) -> Result<#target, crate::form::FormData<Self>> {
                let mut errors: crate::form::FieldErrors = Vec::new();

                #(#field_blocks_update)*

                if !errors.is_empty() {
                    tracing::debug!("Validation errors: {errors:?}");
                    return Err(crate::form::FormData::new_with_errors(
                        self,
                        csrf_tokens,
                        errors,
                    ));
                }

                #[allow(clippy::needless_update)]
                Ok(#target {
                    #(#field_inits,)*
                    ..current.clone()
                })
            }
        }
    }
}

/// Parse the `#[validate(target = "Type")]` attribute from a struct.
fn parse_struct_options(input: &DeriveInput) -> syn::Result<Type> {
    let mut target = None;

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

            Err(meta.error("unsupported validate attribute on struct"))
        })?;
    }

    target.ok_or_else(|| {
        syn::Error::new_spanned(input, "missing #[validate(target = \"Type\")] on struct")
    })
}

/// Parse field-level `#[validate(...)]` options.
///
/// Example:
/// - `#[validate(parse = "Date", optional)]` -> parse `Date`, treat empty string as `None`.
fn parse_field_options(field: &syn::Field) -> syn::Result<FieldOptions> {
    let mut opts = FieldOptions::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("validate") {
            continue;
        }

        attr.parse_nested_meta(|meta| apply_field_option(&mut opts, meta))?;
    }

    Ok(opts)
}

fn apply_field_option(
    opts: &mut FieldOptions,
    meta: syn::meta::ParseNestedMeta,
) -> syn::Result<()> {
    if meta.path.is_ident("optional") {
        return set_optional(opts, &meta);
    }
    if meta.path.is_ident("not_empty") {
        return set_not_empty(opts, &meta);
    }
    if meta.path.is_ident("csrf") {
        return set_csrf(opts, &meta);
    }
    if meta.path.is_ident("ignore") {
        return set_ignore(opts, &meta);
    }
    if meta.path.is_ident("flatten") {
        return set_flatten(opts, &meta);
    }
    if meta.path.is_ident("parse") {
        return set_parse(opts, meta);
    }

    Err(meta.error("unsupported validate attribute on field"))
}

fn set_optional(opts: &mut FieldOptions, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
    if opts.flatten || opts.ignore {
        return Err(meta.error("optional cannot be combined with flatten or ignore"));
    }
    opts.optional = true;
    Ok(())
}

fn set_not_empty(opts: &mut FieldOptions, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
    if opts.not_empty {
        return Err(meta.error("not_empty can only be set once"));
    }
    if opts.flatten || opts.ignore || opts.csrf {
        return Err(meta.error("not_empty cannot be combined with flatten, ignore, or csrf"));
    }
    opts.not_empty = true;
    Ok(())
}

fn set_csrf(opts: &mut FieldOptions, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
    if opts.flatten || opts.ignore || opts.not_empty {
        return Err(meta.error("csrf cannot be combined with flatten, ignore, or not_empty"));
    }
    opts.csrf = true;
    Ok(())
}

fn set_ignore(opts: &mut FieldOptions, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
    if opts.ignore {
        return Err(meta.error("ignore can only be set once"));
    }
    if opts.optional || opts.csrf || opts.parse_ty.is_some() || opts.flatten || opts.not_empty {
        return Err(meta.error("ignore cannot be combined with other validation options"));
    }
    opts.ignore = true;
    Ok(())
}

fn set_flatten(opts: &mut FieldOptions, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
    if opts.flatten {
        return Err(meta.error("flatten can only be set once"));
    }
    if opts.optional || opts.csrf || opts.parse_ty.is_some() || opts.ignore || opts.not_empty {
        return Err(meta.error("flatten cannot be combined with other validation options"));
    }
    opts.flatten = true;
    Ok(())
}

fn set_parse(opts: &mut FieldOptions, meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
    if opts.parse_ty.is_some() {
        return Err(meta.error("only one validator kind is allowed per field"));
    }
    if opts.flatten || opts.ignore {
        return Err(meta.error("parse cannot be combined with flatten or ignore"));
    }
    let lit: LitStr = meta.value()?.parse()?;
    opts.parse_ty = Some(lit.parse::<Type>()?);
    Ok(())
}

struct FieldValidation {
    create_expr: proc_macro2::TokenStream,
    update_expr: proc_macro2::TokenStream,
    validated: bool,
}

/// Dispatch to the correct validation strategy for a field.
///
/// Example:
/// - `#[validate(flatten)]` uses nested validation, otherwise parse or pass-through.
fn build_field_validation(
    ident: &syn::Ident,
    field_name: &str,
    ty: &Type,
    opts: &FieldOptions,
) -> syn::Result<FieldValidation> {
    if opts.flatten {
        return Ok(build_flatten_validation(
            ident,
            field_name,
            is_type_named(ty, "Option"),
        ));
    }

    if let Some(ty) = &opts.parse_ty {
        return Ok(build_parse_validation(ident, field_name, ty, opts.optional));
    }

    Ok(build_passthrough_validation(ident))
}

/// Build validation for a nested form (`#[validate(flatten)]`), forwarding and prefixing errors.
///
/// Example:
/// - `address` field errors become `address.postal_code`.
fn build_flatten_validation(
    ident: &syn::Ident,
    field_name: &str,
    optional: bool,
) -> FieldValidation {
    if optional {
        return build_optional_flatten_validation(ident, field_name);
    }

    let extend_errors = quote! {
        errors.extend(form_data.errors().into_iter().map(|(name, err)| {
            (format!("{}.{}", #field_name, name), err.clone())
        }));
    };
    let create_expr = quote!({
        match self.#ident.clone().validate_create(csrf_tokens) {
            Ok(value) => Some(value),
            Err(form_data) => { #extend_errors None }
        }
    });
    let update_expr = quote!({
        match self.#ident.clone().validate_update(&current.#ident, csrf_tokens) {
            Ok(value) => Some(value),
            Err(form_data) => { #extend_errors None }
        }
    });
    FieldValidation {
        create_expr,
        update_expr,
        validated: true,
    }
}

fn build_optional_flatten_validation(ident: &syn::Ident, field_name: &str) -> FieldValidation {
    let extend_errors = quote! {
        errors.extend(form_data.errors().into_iter().map(|(name, err)| {
            (format!("{}.{}", #field_name, name), err.clone())
        }));
    };
    let create_expr = quote!({
        match self.#ident.clone() {
            Some(value) => match value.validate_create(csrf_tokens) {
                Ok(value) => Some(Some(value)),
                Err(form_data) => { #extend_errors None }
            },
            None => Some(None),
        }
    });
    let update_expr = quote!({
        match self.#ident.clone() {
            Some(value) => match current.#ident.as_ref() {
                Some(current_value) => match value.validate_update(current_value, csrf_tokens) {
                    Ok(value) => Some(Some(value)),
                    Err(form_data) => { #extend_errors None }
                },
                None => match value.validate_create(csrf_tokens) {
                    Ok(value) => Some(Some(value)),
                    Err(form_data) => { #extend_errors None }
                },
            },
            None => Some(None),
        }
    });

    FieldValidation {
        create_expr,
        update_expr,
        validated: true,
    }
}

/// Pass-through validation when no validator is configured.
///
/// Example:
/// - `electoral_districts: Vec<ElectoralDistrict>` is cloned as-is.
fn build_passthrough_validation(ident: &syn::Ident) -> FieldValidation {
    let expr = quote!(self.#ident.clone());
    FieldValidation {
        create_expr: expr.clone(),
        update_expr: expr,
        validated: false,
    }
}

/// Build validation for `#[validate(parse = "...")]` fields.
///
/// Example:
/// - `first_name: String` parsed into `FirstName`.
fn build_parse_validation(
    ident: &syn::Ident,
    field_name: &str,
    ty: &Type,
    optional: bool,
) -> FieldValidation {
    let if_empty = if optional {
        quote!(Some(None))
    } else {
        quote!(
            errors.push((
                #field_name.to_string(),
                crate::form::ValidationError::ValueShouldNotBeEmpty,
            ));
            None
        )
    };

    let res = if optional {
        quote!(Some(Some(value)))
    } else {
        quote!(Some(value))
    };

    let expr = quote!({
        let value = self.#ident.trim();
        if value.is_empty() {
            #if_empty
        } else {
            match <#ty as ::std::str::FromStr>::from_str(value) {
                Ok(value) => #res,
                Err(err) => {
                    errors.push((
                        #field_name.to_string(),
                        crate::form::IntoValidationError::into_validation_error(err)
                    ));
                    None
                }
            }
        }
    });

    FieldValidation {
        create_expr: expr.clone(),
        update_expr: expr,
        validated: true,
    }
}

/// Emit a local binding for a validated field value.
///
/// Example:
/// - `let first_name = <parse expr>;`
fn build_field_block(
    ident: &syn::Ident,
    value_expr: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        let #ident = #value_expr;
    }
}
