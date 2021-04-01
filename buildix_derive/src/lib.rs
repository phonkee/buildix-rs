#[macro_use]
mod select;
mod delete;
mod error;
mod filter;

use error::Error;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use proc_macro_roids::DeriveInputExt;
use quote::quote;

#[proc_macro_derive(DeleteBuilder, attributes(buildix))]
#[proc_macro_error]
pub fn derive_delete_builder(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input);
    let builder: delete::Builder = darling::FromDeriveInput::from_derive_input(&input).unwrap();

    // prepare new tokens
    let mut toks = proc_macro2::TokenStream::new();
    toks.extend(quote! {#builder});
    toks.into()
}

#[proc_macro_derive(Filter, attributes(buildix))]
#[proc_macro_error]
pub fn derive_filter(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input);
    let filter: filter::Filter = darling::FromDeriveInput::from_derive_input(&input).unwrap();

    // prepare new tokens
    let mut toks = proc_macro2::TokenStream::new();
    toks.extend(quote! {#filter});
    toks.into()
}

#[proc_macro_derive(Select, attributes(buildix))]
#[proc_macro_error]
pub fn derive_select(input: TokenStream) -> TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(input);
    let q: select::Select = darling::FromDeriveInput::from_derive_input(&input).unwrap();

    let mut toks = proc_macro2::TokenStream::new();
    toks.extend(quote! {#q});
    toks.into()
}

#[proc_macro_derive(SelectBuilder, attributes(buildix))]
#[proc_macro_error]
pub fn derive_select_builder(input: TokenStream) -> TokenStream {
    let mut input: syn::DeriveInput = syn::parse_macro_input!(input);

    // add derive from sqlx::FromRow (TODO: this is not working currently)
    let derives = syn::parse_quote!(sqlx::FromRow);
    input.append_derives(derives);

    // parse builder
    let sel: select::SelectBuilder = darling::FromDeriveInput::from_derive_input(&input).unwrap();

    // prepare new tokens
    let mut toks = proc_macro2::TokenStream::new();

    toks.extend(quote! {
        #sel
    });

    toks.into()
}
