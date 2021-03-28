#![warn(missing_debug_implementations)]
#![allow(unused_imports)]
#![allow(dead_code)]

use darling::{self, ast, util, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::*;
use quote::quote;
use std::fmt::{Debug, Formatter};

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(buildix),
    supports(struct_named),
    forward_attrs(doc, allow, warn),
    map = "validate_builder"
)]
pub struct Builder {
    // indent
    ident: syn::Ident,

    // data
    data: ast::Data<util::Ignored, BuilderField>,

    #[darling(default)]
    table: String,

    // map function
    #[darling(default)]
    map: Option<syn::Path>,
}

// Builder methods
impl Builder {
    // filter fields by given predicate
    pub fn filter_fields<T>(&self, fun: T) -> Vec<&BuilderField>
    where
        T: Fn(&BuilderField) -> bool,
    {
        self.data
            .as_ref()
            .take_struct()
            .unwrap()
            .fields
            .iter()
            .map(|x| *x)
            .filter(|x| fun(x))
            .collect()
    }

    // list all filter fields
    pub fn list_filter_fields(&self) -> Vec<&BuilderField> {
        self.filter_fields(|x| x.filter)
    }

    // first_field returns first field if available
    pub fn first_field<T>(&self, call: T) -> Option<&BuilderField>
    where
        T: Fn(&BuilderField) -> bool,
    {
        self.filter_fields(call).iter().map(|x| *x).next()
    }

    // validate_single validates if field is single
    pub fn validate_single<T>(&self, call: T, err: crate::Error)
    where
        T: Fn(&BuilderField) -> bool,
    {
        if let Some(invalid) = self.filter_fields(call).iter().skip(1).next() {
            abort!(invalid.ident, err);
        }
    }
}

// validate delete builder
fn validate_builder(builder: Builder) -> Builder {
    let mut builder = builder;
    builder.table = builder.table.trim().clone().to_string();

    if builder.table.is_empty() {
        abort!(builder.ident, r#"Please provide `#[buildix(table=...)]`"#);
    }

    if builder.list_filter_fields().is_empty() {
        abort!(
            builder.ident,
            r#"Please provide at least one `#[buildix(filter)]` field"#
        );
    }

    // validate single limit
    builder.validate_single(
        |x| x.limit,
        crate::error::Error::Multiple("#[buildix(limit)]".to_string()),
    );

    builder
}

#[derive(Clone, Debug, FromField)]
#[darling(attributes(buildix), forward_attrs(doc, allow, warn))]
pub struct BuilderField {
    // field name
    pub ident: Option<syn::Ident>,

    // field type
    pub ty: syn::Type,

    #[darling(default)]
    count: bool,

    #[darling(default)]
    filter: bool,

    #[darling(default)]
    limit: bool,
}

// validate_field validates single field
fn validate_field(f: BuilderField) -> BuilderField {
    // let mut f = f;

    // check if we have set at least one argument
    if ![f.count, f.filter, f.limit].iter().any(|x| *x) {
        abort!(f.ident.unwrap(), crate::Error::InvalidDelete);
    }

    f
}

// generate code for delete builder
impl quote::ToTokens for Builder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        tokens.extend(quote! {
            use buildix::delete::DeleteBuilder as __DeleteBuilder;
            use sqlx::Database;


            // implement DeleteBuilder
            impl ::buildix::DeleteBuilder for #ident {

                // generate sql along with arguments
                fn to_sql<DB: Database>(&mut self) -> buildix::Result<(String, Vec<()>)> {
                    Ok(("DELETE FROM what WHERE 1 = 1".to_string(), vec![]))
                }
            }
        });
    }
}
