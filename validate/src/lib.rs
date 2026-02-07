use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, Type, parse_macro_input};

/// Derive `Validate` implementations with field annotations.
///
/// Supported annotations:
/// - `#[validate(target = "Type")]` on the struct.
/// - `#[validate(target = "Type", timestamps = false)]` to skip auto timestamps.
/// - `#[validate(parse = "Type")]` to parse via `Type::from_str`.
/// - `#[validate(optional)]` to treat empty strings as `None`.
/// - `#[validate(csrf)]` to validate CSRF tokens.
/// - `#[validate(ignore)]` to skip validation and mapping for a field.
/// - `#[validate(flatten)]` to validate a nested form and prefix its errors with `field[child]`.
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
    timestamps: bool,
}

#[derive(Default)]
struct FieldOptions {
    optional: bool,
    csrf: bool,
    ignore: bool,
    validator: Option<Validator>,
    flatten: bool,
}

enum Validator {
    Parse { ty: Type },
}

/// Expand the `Validate` derive into an implementation for the target type.
///
/// Example:
/// - `PersonForm` + `#[validate(target = "Person")]` -> `impl Validate<Person> for PersonForm`.
fn expand_validate(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_options = parse_struct_options(input)?;
    let struct_name = &input.ident;
    let target = struct_options.target;
    let timestamps = struct_options.timestamps;

    let fields = collect_named_fields(input)?;
    let field_blocks = build_field_blocks(&fields)?;
    let (create_timestamps, update_timestamps) = build_timestamp_tokens(timestamps);
    let with_csrf_impl = build_with_csrf_impl(struct_name, field_blocks.has_csrf);
    let tokens = build_validate_impl(
        struct_name,
        &target,
        &field_blocks,
        &create_timestamps,
        &update_timestamps,
        with_csrf_impl,
    );

    Ok(tokens.into())
}

struct FieldBlocks {
    field_inits: Vec<proc_macro2::TokenStream>,
    field_blocks_create: Vec<proc_macro2::TokenStream>,
    field_blocks_update: Vec<proc_macro2::TokenStream>,
    has_csrf: bool,
}

fn collect_named_fields<'a>(input: &'a DeriveInput) -> syn::Result<Vec<&'a syn::Field>> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(fields.named.iter().collect::<Vec<_>>()),
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

        let validation = build_field_validation(ident, &field_name, &opts)?;

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

fn build_timestamp_tokens(
    timestamps: bool,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let create_timestamps = if timestamps {
        quote! {
            updated_at: crate::UtcDateTime::now(),
            created_at: crate::UtcDateTime::now(),
        }
    } else {
        quote! {}
    };

    let update_timestamps = if timestamps {
        quote! {
            updated_at: crate::UtcDateTime::now(),
        }
    } else {
        quote! {}
    };

    (create_timestamps, update_timestamps)
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
    create_timestamps: &proc_macro2::TokenStream,
    update_timestamps: &proc_macro2::TokenStream,
    with_csrf_impl: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let field_inits = &field_blocks.field_inits;
    let field_blocks_create = &field_blocks.field_blocks_create;
    let field_blocks_update = &field_blocks.field_blocks_update;

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
                    #create_timestamps
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
                    #update_timestamps
                    ..current.clone()
                })
            }
        }
    }
}

/// Parse struct-level `#[validate(...)]` options.
///
/// Example:
/// - `#[validate(target = "Person", timestamps = false)]` -> target `Person`, timestamps disabled.
fn parse_struct_options(input: &DeriveInput) -> syn::Result<StructOptions> {
    let mut target = None;
    let mut timestamps = true;

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
            if meta.path.is_ident("timestamps") {
                let lit: syn::LitBool = meta.value()?.parse()?;
                timestamps = lit.value;
                return Ok(());
            }

            Err(meta.error("unsupported validate attribute on struct"))
        })?;
    }

    let target = target.ok_or_else(|| {
        syn::Error::new_spanned(input, "missing #[validate(target = \"Type\")] on struct")
    })?;

    Ok(StructOptions { target, timestamps })
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

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("optional") {
                if opts.flatten || opts.ignore {
                    return Err(meta.error("optional cannot be combined with flatten or ignore"));
                }
                opts.optional = true;
                return Ok(());
            }
            if meta.path.is_ident("csrf") {
                if opts.flatten || opts.ignore {
                    return Err(meta.error("csrf cannot be combined with flatten or ignore"));
                }
                opts.csrf = true;
                return Ok(());
            }
            if meta.path.is_ident("ignore") {
                if opts.ignore {
                    return Err(meta.error("ignore can only be set once"));
                }
                if opts.optional || opts.csrf || opts.validator.is_some() || opts.flatten {
                    return Err(
                        meta.error("ignore cannot be combined with other validation options")
                    );
                }
                opts.ignore = true;
                return Ok(());
            }
            if meta.path.is_ident("flatten") {
                if opts.flatten {
                    return Err(meta.error("flatten can only be set once"));
                }
                if opts.optional || opts.csrf || opts.validator.is_some() || opts.ignore {
                    return Err(
                        meta.error("flatten cannot be combined with other validation options")
                    );
                }
                opts.flatten = true;
                return Ok(());
            }
            if meta.path.is_ident("parse") {
                if opts.validator.is_some() {
                    return Err(meta.error("only one validator kind is allowed per field"));
                }
                if opts.flatten || opts.ignore {
                    return Err(meta.error("parse cannot be combined with flatten or ignore"));
                }
                let lit: LitStr = meta.value()?.parse()?;
                let ty = lit.parse::<Type>()?;
                opts.validator = Some(Validator::Parse { ty });
                return Ok(());
            }

            Err(meta.error("unsupported validate attribute on field"))
        })?;
    }

    Ok(opts)
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
    opts: &FieldOptions,
) -> syn::Result<FieldValidation> {
    if opts.flatten {
        return Ok(build_flatten_validation(ident, field_name));
    }

    let Some(validator) = &opts.validator else {
        return Ok(build_passthrough_validation(ident));
    };

    match validator {
        Validator::Parse { ty } => Ok(build_parse_validation(ident, field_name, ty, opts.optional)),
    }
}

/// Build validation for a nested form (`#[validate(flatten)]`), forwarding and prefixing errors.
///
/// Example:
/// - `address` field errors become `address.postal_code`.
fn build_flatten_validation(ident: &syn::Ident, field_name: &str) -> FieldValidation {
    let prefix = field_name.to_string();
    let create_expr = quote!({
        match self.#ident.clone().validate_create(csrf_tokens) {
            Ok(value) => Some(value),
            Err(form_data) => {
                errors.extend(form_data.errors().into_iter().map(|(name, err)| {
                    (format!("{}.{}", #prefix, name), err.clone())
                }));
                None
            }
        }
    });
    let update_expr = quote!({
        match self.#ident.clone().validate_update(&current.#ident, csrf_tokens) {
            Ok(value) => Some(value),
            Err(form_data) => {
                errors.extend(form_data.errors().into_iter().map(|(name, err)| {
                    (format!("{}.{}", #prefix, name), err.clone())
                }));
                None
            }
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
    FieldValidation {
        create_expr: quote!(self.#ident.clone()),
        update_expr: quote!(self.#ident.clone()),
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
    let expr = if optional {
        build_optional_parse_expr(ident, field_name, ty)
    } else {
        build_required_parse_expr(ident, field_name, ty)
    };

    FieldValidation {
        create_expr: expr.clone(),
        update_expr: expr,
        validated: true,
    }
}

/// Build parse expression for optional fields (empty string => `None`).
///
/// Example:
/// - `country_code: String` -> `Option<CountryCode>`.
fn build_optional_parse_expr(
    ident: &syn::Ident,
    field_name: &str,
    ty: &Type,
) -> proc_macro2::TokenStream {
    quote!({
        let value = self.#ident.trim();
        if self.#ident.is_empty() {
            Some(None)
        } else {
            match <#ty as ::std::str::FromStr>::from_str(value) {
                Ok(value) => Some(Some(value)),
                Err(err) => {
                    errors.push((
                        #field_name.to_string(),
                        crate::form::IntoValidationError::into_validation_error(err)
                    ));
                    None
                }
            }
        }
    })
}

/// Build parse expression for required fields (empty string => error).
fn build_required_parse_expr(
    ident: &syn::Ident,
    field_name: &str,
    ty: &Type,
) -> proc_macro2::TokenStream {
    quote!({
        let value = self.#ident.trim();
        if value.is_empty() {
            errors.push((
                #field_name.to_string(),
                crate::form::ValidationError::ValueShouldNotBeEmpty,
            ));

            None
        } else {
            match <#ty as ::std::str::FromStr>::from_str(value) {
                Ok(value) => Some(value),
                Err(err) => {
                    errors.push((
                        #field_name.to_string(),
                        crate::form::IntoValidationError::into_validation_error(err)
                    ));
                    None
                }
            }
        }
    })
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
