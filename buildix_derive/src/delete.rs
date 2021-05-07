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

    // write_simple_query writes `get_simple_query` method
    pub fn write_get_simple_query(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let simple_query = format!("DELETE FROM {}", self.table);
        tokens.extend(quote! {
            impl #ident {
                fn get_simple_query(&self) -> &'static str {
                    #simple_query
                }
            }
        });
    }

    // write filter
    pub fn write_filter(&self, _tokens: &mut TokenStream, _target_string: &syn::Ident) {
        // now write filter implementation

        // prepare fields
        let fields: Vec<crate::filter::process::Field> = self
            .list_filter_fields()
            .iter()
            .map(|x| x.clone())
            .map(|f| f.into())
            .collect();

        // process filter
        crate::filter::process::process(&self.ident, fields, " AND ".to_string(), _tokens)
    }

    // write limit
    pub fn write_limit(&self, tokens: &mut TokenStream, target_string: &syn::Ident) {
        if let Some(field) = self.first_field(|x| x.limit) {
            let ident = field.ident.as_ref().unwrap();

            tokens.extend(quote! {
                if let Some(clause) = self.#ident.get_limit::<DB>() {
                    #target_string.push_str(" ");
                    #target_string.push_str(&clause);
                }
            });
        }
    }

    // write map implementation
    pub fn write_map(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        if let Some(path) = &self.map {
            tokens.extend(quote! {
                let _fun: &dyn Fn(&mut #ident) -> buildix::Result<()> = &#path;

                // call map function
                let _ = #path(self)?;
            });
        }
    }
}

// generate code for delete builder
impl quote::ToTokens for Builder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // prepare ident
        let ident = &self.ident;

        // all assertions (fields and other)
        let mut target = TokenStream::new();

        // write `get_simple_query`
        self.write_get_simple_query(&mut target);

        // prepare all assertions
        for field in self.filter_fields(|_| true) {
            field.write_assertions(&mut target);
        }

        let mut limit_impl = TokenStream::new();

        // prepare query ident (string to append values)
        let final_query_ident = syn::Ident::new("query", Span::call_site());

        // write filter now
        self.write_filter(&mut target, &final_query_ident);
        self.write_limit(&mut limit_impl, &final_query_ident);

        // now do map implementation
        let mut map_impl = TokenStream::new();
        self.write_map(&mut map_impl);

        // extend tokens with additional implementations
        tokens.extend(quote! {
            use buildix::delete::DeleteBuilder as __DeleteBuilder;
            use sqlx::Database;
            use static_assertions;
            use buildix::Filter as __Filter;
            use buildix::limit::Limit as __Limit;

            // add all assertions for compiler
            #target

            // implement DeleteBuilder
            impl ::buildix::DeleteBuilder for #ident {

                // generate sql along with arguments
                fn to_sql<DB: Database>(&mut self) -> buildix::Result<(String, ::sqlx::any::AnyArguments)> {

                    // check map now

                    // prepare query
                    let mut query: String = self.get_simple_query().to_owned();

                    // now process filter
                    let fi = buildix::filter::FilterInfo::default();
                    if let Some(_filter_result) = self.filter_query::<DB>(&fi) {
                        // query.push_str(" WHERE ");
                        // query.push_str(&filter_result.clause);
                    }

                    // now limit
                    #limit_impl

                    Ok((query, ::sqlx::any::AnyArguments::default()))
                }
            }

            // Here comes execute

        });
    }
}

/// validate delete builder
/// we need to be sure that following rules apply:
///     * we have at least one filter field
///     * we have at most one limit field
///     * we have at most one count field
///     * we have `table` set
fn validate_builder(builder: Builder) -> Builder {
    let mut builder = builder;
    builder.table = builder.table.trim().clone().to_string();

    // check if we have table set
    if builder.table.is_empty() {
        abort!(builder.ident, r#"Please provide `#[buildix(table=...)]`"#);
    }

    // check if we have filter field - good practice to be sure that we don't have wild builders
    if builder.first_field(|x| x.filter).is_none() {
        abort!(
            builder.ident,
            r#"Please provide at least one `#[buildix(filter)]` field"#
        );
    }

    // validate if we have single limit
    builder.validate_single(
        |x| x.limit,
        crate::error::Error::MultipleFields("#[buildix(limit)]".to_string()),
    );

    // validate if we have single limit
    builder.validate_single(
        |x| x.count,
        crate::error::Error::MultipleFields("#[buildix(count)]".to_string()),
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

impl From<&BuilderField> for crate::filter::process::Field {
    fn from(f: &BuilderField) -> Self {
        Self {
            ident: f.ident.as_ref().unwrap().clone(),
            ty: f.ty.clone(),
            expr: "".to_string(),
            table: "".to_string(),
            column: f.ident.as_ref().unwrap().to_string(),
            isnull: false,
        }
    }
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

// implement delete builder field
impl BuilderField {
    // write assertions based on field
    pub fn write_assertions(&self, tokens: &mut TokenStream) {
        let ty = &self.ty;
        if self.count {
            tokens.extend(quote! {
                static_assertions::assert_impl_all!(#ty: ::buildix::Count);
            });
        }
        if self.limit {
            tokens.extend(quote! {
                static_assertions::assert_impl_all!(#ty: ::buildix::Limit);
            });
        }
        if self.filter {
            tokens.extend(quote! {
                static_assertions::assert_impl_all!(#ty: ::buildix::Filter);
            });
        }
    }
}
