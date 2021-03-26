#![allow(dead_code)]
#![allow(unused_imports)]
#![warn(missing_debug_implementations)]

pub mod process;

use darling::{self, ast, util, FromDeriveInput, FromField, FromMeta};
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(buildix),
    supports(struct_named),
    forward_attrs(doc, allow, warn),
    map = "validate_filter"
)]
pub struct Filter {
    ident: syn::Ident,

    #[darling(default)]
    expr: String,

    #[darling(default)]
    operator: Operator,

    // data
    data: ast::Data<util::Ignored, Field>,

    // passed attrs
    attrs: Vec<syn::Attribute>,

    // map function to validate filter
    #[darling(default)]
    map: Option<syn::Path>,
}

#[derive(Debug, FromMeta, PartialEq, Eq)]
struct Operator(String);

impl Default for Operator {
    fn default() -> Self {
        Operator("AND".to_owned())
    }
}

impl quote::ToTokens for Filter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields: Vec<process::Field> = self
            .data
            .as_ref()
            .take_struct()
            .unwrap()
            .fields
            .iter()
            .map(|f| *f)
            .map(|f| f.into())
            .collect();

        // first do some compile time assertions
        for field in self.data.as_ref().take_struct().unwrap().fields {
            field.to_tokens(tokens);
        }

        // process
        process::process(&self.ident, fields, self.operator.0.clone(), tokens);
    }
}

// validate filter
pub fn validate_filter(f: Filter) -> Filter {
    // validate operator first
    f
}

#[derive(Debug, FromField)]
#[darling(
    attributes(buildix),
    forward_attrs(doc, allow, warn),
    map = "map_field"
)]
pub struct Field {
    pub ident: Option<syn::Ident>,

    // field type
    pub ty: syn::Type,

    #[darling(default)]
    pub expr: String,

    #[darling(default)]
    pub table: String,

    #[darling(default)]
    pub column: String,

    #[darling(default)]
    pub isnull: bool,
}

impl quote::ToTokens for Field {
    // TODO: assertions
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        // now abort when anything is wrong

        // abort!(self.ident, "error");
    }
}

fn map_field(f: Field) -> Field {
    let mut f = f;
    f.expr = f.expr.trim().to_string();
    f.table = f.table.trim().to_string();
    f.column = f.column.trim().to_string();

    f
}
