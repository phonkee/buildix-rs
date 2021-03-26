#![warn(missing_debug_implementations)]
#![allow(unused_imports)]

use crate::error::Error;
use darling::{self, ast, util, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::{format_ident, quote};

#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(buildix),
    supports(struct_named),
    forward_attrs(doc, allow, warn),
    map = "validate"
)]
pub struct Select {
    // indent
    ident: syn::Ident,

    // data
    data: ast::Data<util::Ignored, Field>,

    // passed attrs
    attrs: Vec<syn::Attribute>,

    #[darling(default, multiple, rename = "from")]
    froms: Vec<FromAttribute>,

    #[darling(default, multiple)]
    group: Vec<String>,
}

#[derive(Debug, FromField)]
#[darling(
    attributes(buildix),
    forward_attrs(doc, allow, warn),
    map = "map_field"
)]
pub struct Field {
    // field name
    pub ident: Option<syn::Ident>,

    // field type
    pub ty: syn::Type,

    #[darling(default)]
    pub expr: String,

    #[darling(default)]
    pub table: String,

    #[darling(default)]
    pub column: String,
}

// map_field is called before init struct, we can do our checks and prepare `column` which is
// the final thing that can be used safely
fn map_field(f: Field) -> Field {
    let mut f = f;

    // first trim off whitespace
    f.expr = f.expr.trim().to_string();
    f.table = f.table.trim().to_string();
    f.column = f.column.trim().to_string();

    let original_ident = f.ident.as_ref().unwrap().to_string();

    // validate
    if !f.expr.is_empty() && !f.table.is_empty() {
        abort!(f.ident.as_ref().unwrap(), Error::InvalidSelectField)
    }

    let mut what = "".to_string();

    if f.expr.is_empty() {
        if !f.column.is_empty() {
            f.expr = f.column.clone();
        } else {
            f.expr = original_ident.clone()
        }
        what = f.expr.clone();

        if !f.table.is_empty() {
            f.expr = format!("{}.{}", f.table, f.expr);
        }
    }

    if what != original_ident {
        f.expr = format!("{} AS {}", f.expr, original_ident);
    }

    f
}

#[derive(Debug, FromMeta)]
#[darling(rename_all = "snake_case")]
#[darling(map = "map_from_attribute")]
pub enum FromAttribute {
    Table {
        #[darling(default)]
        name: String,

        #[darling(default)]
        alias: String,
    },
    Join {
        #[darling(default)]
        name: String,

        #[darling(default)]
        alias: String,

        #[darling(default, rename = "on")]
        join_on: String,

        #[darling(default)]
        join_type: String,
    },
}

// map_from_attribute should trim any strings and do some housekeeping
fn map_from_attribute(t: FromAttribute) -> FromAttribute {
    // let mut t = t;
    // t.name = t.name.trim().to_owned();
    // t.alias = t.alias.trim().to_owned();
    // t.join = t.join.trim().to_owned();
    // t.join_on = t.join_on.trim().to_owned();
    // t.join_type = t.join_type.trim().to_owned();
    t
}

impl Select {
    pub fn get_group_by(&self) -> Option<String> {
        if self.group.is_empty() {
            None
        } else {
            Some(format!("GROUP BY {}", self.group.join(", ")))
        }
    }

    // get table names
    pub fn get_from_tables(&self, _ident: &syn::Ident) -> String {
        let mut wheres: Vec<String> = Vec::with_capacity(self.froms.len());

        for sel in &self.froms {
            wheres.push(match sel {
                FromAttribute::Table { name, alias } => {
                    let alias = alias.trim();
                    if !alias.is_empty() {
                        format!("{} AS {}", name, alias)
                    } else {
                        name.clone()
                    }
                }
                FromAttribute::Join {
                    name,
                    alias,
                    join_on,
                    join_type,
                } => {
                    let mut join_type = join_type.trim().to_string();
                    let mut name = name.trim().to_string();
                    let alias = alias.trim().to_string();
                    if !alias.is_empty() {
                        name = format!("{} {}", name, alias).to_string();
                    }

                    if join_type.is_empty() {
                        join_type = "inner join".to_uppercase().to_owned()
                    }
                    if !join_type.to_lowercase().contains("join") {
                        join_type = format!("{} join", join_type).to_uppercase();
                    }

                    let mut join_on = join_on.clone();
                    if !join_on.is_empty() {
                        join_on = format!("({})", join_on);
                    }

                    format!("{} {} {}", join_type, name, join_on)
                        .trim()
                        .to_string()
                }
            })
        }
        wheres.join(", ")
    }
}

impl Select {
    // get_fields returns all fields
    pub fn get_fields(&self) -> Vec<String> {
        vec![]
    }
}

// generate tokens from query
impl quote::ToTokens for Select {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        let ident = &self.ident;

        // list over all
        let fields: Vec<String> = self
            .data
            .as_ref()
            .take_struct()
            .unwrap()
            .fields
            .iter()
            .map(|f| f.expr.clone())
            .collect();

        let all_fields = fields.join(", ");
        let table = self.get_from_tables(&self.ident);
        let query = format!("SELECT {} FROM {}", all_fields, table);

        let mut group_tokens = TokenStream::new();

        if let Some(group_by) = self.get_group_by() {
            group_tokens.extend(quote! {
                Some(#group_by)
            });
        } else {
            group_tokens.extend(quote! {
                None
            });
        }

        _tokens.extend(quote! {
            #[allow(unused_imports)]
            use buildix::Select as __Select;

            // implement query first
            impl ::buildix::Select for #ident {
                #[inline]
                fn get_fields_str(&self) -> &'static str {
                    #all_fields
                }
                #[inline]
                fn get_table(&self) -> &'static str {
                    #table
                }
                #[inline]
                fn get_fields(&self) -> &'static [&'static str] {
                    &[
                        #(#fields),*
                    ]
                }
                #[inline]
                fn get_query(&self) -> &'static str {
                    #query
                }
                #[inline]
                fn get_group(&mut self) -> Option<&'static str> {
                    #group_tokens
                }
            }

            // implement Select
            // impl ::buildix::select::Select for #ident {
                // const BASIC_QUERY: &'static str = #base_query;
            // }
        })
    }
}

// validate query
fn validate(q: Select) -> Select {
    // let mut q = q;
    // if q.tables.is_empty() {
    //     q.tables.push(Table::default())
    // }
    //
    // for mut t in &mut q.tables {
    //     // verify if name is provided
    //     if t.name.is_empty() && t.join.is_empty() {
    //         t.name = ident_case::RenameRule::SnakeCase.apply_to_variant(q.ident.to_string());
    //     }
    // }
    //
    q
}
