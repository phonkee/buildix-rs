use crate::error::Error;
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::quote;

#[derive(Debug)]
pub struct Field {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub expr: String,
    pub table: String,
    pub column: String,
    pub isnull: bool,
}

impl Field {
    pub fn get_expr(&self) -> String {
        let mut expr = self.expr.trim().to_string();

        if expr.is_empty() {
            let mut ident = self.ident.to_string();
            if !self.column.is_empty() {
                ident = self.column.clone();
            }

            if !self.table.is_empty() {
                ident = format!("{}.{}", self.table, ident);
            }

            if self.expr.is_empty() {
                expr = format!("{} = ?", ident);
            }
        }

        expr
    }
}

impl From<&crate::filter::Field> for Field {
    fn from(out: &crate::filter::Field) -> Self {
        Self {
            ident: out.ident.clone().unwrap(),
            ty: out.ty.clone(),
            expr: out.expr.clone(),
            isnull: out.isnull,
            table: out.table.clone(),
            column: out.column.clone(),
        }
    }
}

impl From<&crate::select::field::Field> for Field {
    fn from(out: &crate::select::field::Field) -> Self {
        Self {
            ident: out.ident.clone().unwrap(),
            ty: out.ty.clone(),
            expr: "".to_string(),
            isnull: false,
            table: "".to_string(),
            column: "".to_string(),
        }
    }
}

// process fields and write implementation
pub fn process(ident: &syn::Ident, fields: Vec<Field>, operator: String, tokens: &mut TokenStream) {
    let mut field_asserts = TokenStream::new();
    let mut field_impl = TokenStream::new();

    for field in &fields {
        let field_type = &field.ty;
        let field_ident = &field.ident;
        let field_ident_str = field_ident.to_string();
        let field_expr = field.get_expr();
        let has_field_expr = !field_expr.is_empty();

        field_asserts.extend(quote! {
            static_assertions::assert_impl_all!(#field_type: ::buildix::filter::Filter);
        });

        // check if this field can be nullable
        if field.isnull {
            field_asserts.extend(quote! {
                static_assertions::assert_impl_all!(#field_type: ::buildix::filter::Nullable);
            });
        }

        let mut expr_tokens = TokenStream::new();
        if has_field_expr {
            expr_tokens.extend(quote! {
                filter_info.expr = Some(#field_expr.to_string());
            });
        }

        let isnull = field.isnull;

        // add actual implementation
        field_impl.extend(quote! {
            // set values
            filter_info.ident = #field_ident_str;
            filter_info.isnull = #isnull;

            #expr_tokens

            // call process_filter
            if let Some(clause) = self.#field_ident.filter_query::<DB>(&filter_info) {
                // check if we have filters (count)
                filter_clauses.push(clause);
            };
        });
    }

    let operator = format!(" {} ", operator);

    // generate filter stuff
    tokens.extend(quote! {
        use buildix::prelude::*;
        use sqlx::Database as _;
        use sqlx::query::QueryAs as _;
        use sqlx::IntoArguments as _;

        // field assertions first
        #field_asserts

        // filter implementation
        impl ::buildix::filter::Filter for #ident {

            fn filter_query<DB: Database>(&self, info: &::buildix::filter::FilterInfo) -> Option<String> {
                let mut filter_clauses: Vec<String> = vec![];
                let mut filter_info = ::buildix::filter::FilterInfo::default();

                #field_impl

                // check for clauses
                if filter_clauses.is_empty() {
                    None
                } else {
                    // get size of values
                    let mut clause = filter_clauses.join(#operator);

                    if filter_clauses.len() > 1 {
                        clause = format!("({})", clause);
                    }

                    Some(clause)
                }
            }

            // filter arguments
            fn filter_arguments<'q, DB, O, T>(&self, query: ::sqlx::query::QueryAs<'q, DB, O, T>) -> ::sqlx::query::QueryAs<'q, DB, O, T>
            where
                DB: Database,
                T: ::sqlx::IntoArguments<'q, DB>,
            {
                println!("filter_arguments");
                query
            }
        }
    });
}
