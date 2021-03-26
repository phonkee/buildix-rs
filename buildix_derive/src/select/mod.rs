#![warn(missing_debug_implementations)]
#![allow(unused_imports)]
#![allow(dead_code)]

pub mod field;
mod select;

pub use select::Select;

use darling::{self, ast, util, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(buildix),
    supports(struct_named),
    forward_attrs(doc, allow, warn),
    map = "validate"
)]
pub struct SelectBuilder {
    // indent
    ident: syn::Ident,

    // data
    data: ast::Data<util::Ignored, field::Field>,

    // map function
    #[darling(default)]
    map: Option<syn::Path>,
}

impl SelectBuilder {
    // validate_single validates if field is single
    pub fn validate_single<T>(&self, call: T, err: crate::Error)
    where
        T: Fn(&field::Field) -> bool,
    {
        // check for duplicate query
        if let Some(invalid) = self
            .data
            .as_ref()
            .take_struct()
            .unwrap()
            .fields
            .iter()
            .filter(|x| call(x))
            .skip(1)
            .next()
        {
            abort!(invalid.ident, err);
        };
    }
    pub fn filter_fields<T>(&self, call: T) -> Vec<&field::Field>
    where
        T: Fn(&field::Field) -> bool,
    {
        self.data
            .as_ref()
            .take_struct()
            .unwrap()
            .fields
            .iter()
            .map(|x| *x)
            .filter(|x| call(x))
            .collect()
    }

    // get_first_field returns first field
    pub fn get_first_field<T>(&self, call: T) -> Option<&field::Field>
    where
        T: Fn(&field::Field) -> bool,
    {
        self.filter_fields(|x| call(x)).iter().map(|x| *x).next()
    }

    // we are totally sure that we have at least one field
    pub fn get_select_field(&self) -> &field::Field {
        self.get_first_field(|x| x.select).unwrap()
    }

    // get limit field
    pub fn get_limit_field(&self) -> Option<&field::Field> {
        self.get_first_field(|x| x.limit)
    }

    // get offset field
    pub fn get_offset_field(&self) -> Option<&field::Field> {
        self.get_first_field(|x| x.offset)
    }

    // get group field
    pub fn get_group_field(&self) -> Option<&field::Field> {
        self.get_first_field(|x| x.group)
    }

    // get sort field
    pub fn get_sort_fields(&self) -> Vec<&field::Field> {
        self.filter_fields(|x| x.sort.is_some())
    }

    // list all filter fields
    pub fn get_filter_fields(&self) -> Vec<&field::Field> {
        self.filter_fields(|x| x.filter)
    }

    // list all having
    pub fn get_having_fields(&self) -> Vec<&field::Field> {
        self.filter_fields(|x| x.having)
    }
}

// generate tokens from select
impl quote::ToTokens for SelectBuilder {
    // write tokens from select
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        let ident = &self.ident;
        let select_field_type = &self.get_select_field().ty;
        let select_field_ident = &self.get_select_field().ident;

        // prepare sort field
        let mut sort_tokens = TokenStream::new();
        let mut sort_tokens_asserts = TokenStream::new();
        let mut sort_clause = TokenStream::new();

        let sorts_len = self.get_sort_fields().len();

        // iterate over fields
        for field in self.get_sort_fields() {
            let sort_ident = field.ident.as_ref().unwrap();
            let sort_ident_db = &field.sort.as_ref().unwrap();
            let sort_ty = &field.ty;
            sort_tokens_asserts.extend(quote! {
                static_assertions::assert_impl_all!(#sort_ty: ::buildix::sort::Sorter);
            });

            // now do something
            sort_tokens.extend(quote! {
                if let Some(sort_clause) = self.#sort_ident.sort(#sort_ident_db) {
                    sorts.push(sort_clause);
                }
            });
        }

        // add sorts if available
        if sorts_len > 0 {
            sort_clause.extend(quote! {
                use buildix::sort::Sorter;
                let mut sorts: Vec<String> = Vec::with_capacity(#sorts_len);
                #sort_tokens
                if sorts.len() > 0 {
                    parts.push(format!("ORDER BY {}", sorts.join(", ")));
                }
            });
        }

        // add offset and limit
        let limit_field = &self.get_limit_field();
        let offset_field = &self.get_offset_field();

        // prepare limit offset
        let mut limit_offset_clause = TokenStream::new();
        let mut limit_offset_assert = TokenStream::new();

        // offset cannot be used without limit, so first is to check limit
        if let Some(limit_field) = limit_field {
            let limit_field_type = &limit_field.ty;
            let limit_field_ident = &limit_field.ident.as_ref();
            // assert limit type
            limit_offset_assert.extend(quote! {
                static_assertions::assert_impl_all!(#limit_field_type: ::buildix::limit::Limit);
            });

            let mut offset_clause = TokenStream::new();

            if let Some(offset_field) = offset_field {
                let offset_field_type = &offset_field.ty;
                let offset_field_ident = &offset_field.ident.as_ref();
                limit_offset_assert.extend(quote! {
                    static_assertions::assert_impl_all!(#offset_field_type: ::buildix::offset::Offset);
                });

                offset_clause.extend(quote! {
                    if let Some(clause) = self.#offset_field_ident.get_offset() {
                        parts.push(clause);
                    }
                });
            }

            limit_offset_clause.extend(quote! {
                if let Some(clause) = self.#limit_field_ident.get_limit() {
                    parts.push(clause);

                    // add offset if available
                    #offset_clause
                }
            });
        }

        // now do filter
        let mut filter_tokens = TokenStream::new();
        let filter_fields: Vec<crate::filter::process::Field> = self
            .get_filter_fields()
            .iter()
            .map(|f| *f)
            .map(|f| f.into())
            .collect();

        // now create filter
        crate::filter::process::process(
            &self.ident,
            filter_fields,
            "AND".to_string(),
            &mut filter_tokens,
        );

        // generate traits for select
        _tokens.extend(quote! {
            #[allow(unused_imports)]
            use buildix::SelectBuilder as __SelectBuilder;
            use buildix::sort::Sort as __Sort;
            use buildix::offset::Offset as __Offset;
            use buildix::limit::Limit as __Limit;
            use buildix::filter::Filter as __Filter;
            use buildix::prelude::*;
            use static_assertions;

            #limit_offset_assert
            #sort_tokens_asserts

            // filter implementation
            #filter_tokens

            // implement Select
            impl ::buildix::SelectBuilder for #ident {
                // get_query returns query string
                fn get_query(&mut self) -> (String, Vec<()>) {
                    // prepare query
                    // first is base query which should be prepared in binary
                    let mut parts: Vec<String> = vec![self.#select_field_ident.get_query().to_string()];

                    // GROUP BY
                    if let Some(group_by) = self.#select_field_ident.get_group() {
                        parts.push(group_by.to_owned());
                    }

                    #sort_clause
                    #limit_offset_clause

                    // now
                    let values: Vec<()> = vec![];

                    let fi = buildix::filter::FilterInfo::default();
                    if let Some(filter_result) = self.process_filter(&fi) {
                        if !filter_result.clause.is_empty() {
                            parts.push(format!("WHERE {}", filter_result.clause).to_string());
                        }
                    }

                    let query = parts.join(" ");

                    (query, values)
                }
            }

            // assert that everything is fine
            static_assertions::assert_impl_all!(#select_field_type: ::buildix::Select);

            // impl ::buildix::execute::Execute for #ident {
            //     fn execute(
            //         &mut self,
            //         _pool: ::sqlx::Pool<::sqlx::postgres::Postgres>,
            //     ) -> std::future::Future<Output = Result<(), ::sqlx::Error>> {
            //         Ok(())
            //     }
            // }
        })
    }
}

// validate Select
pub fn validate(s: SelectBuilder) -> SelectBuilder {
    let mut found_query = false;

    // check if #[query] is missing
    s.data.as_ref().map_struct_fields(|f| {
        let f_ident = f.ident.as_ref().unwrap();

        // found query
        if f.select {
            found_query = true
        }

        if let Err(ref err) = f.validate() {
            abort!(f_ident, err);
        };
    });

    // check if we have query
    if !found_query {
        abort!(s.ident, crate::Error::MissingQuery);
    }

    // check for duplicates
    s.validate_single(
        |f| f.select,
        crate::Error::Multiple("#[buildix(query)]".to_string()),
    );
    s.validate_single(
        |f| f.offset,
        crate::Error::Multiple("#[buildix(offset)]".to_string()),
    );
    s.validate_single(
        |f| f.limit,
        crate::Error::Multiple("#[buildix(limit)]".to_string()),
    );
    // check for duplicate group
    s.validate_single(
        |f| f.group,
        crate::Error::Multiple("#[buildix(group)]".to_string()),
    );

    s
}
