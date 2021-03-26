use darling::FromField;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone, Debug, FromField)]
#[darling(attributes(buildix), forward_attrs(doc, allow, warn))]
pub struct Field {
    // field name
    pub(crate) ident: Option<syn::Ident>,

    // field type
    pub(crate) ty: syn::Type,

    #[darling(default)]
    pub(crate) select: bool,

    #[darling(default)]
    pub(crate) filter: bool,

    #[darling(default)]
    pub(crate) offset: bool,

    #[darling(default)]
    pub(crate) limit: bool,

    #[darling(default)]
    pub(crate) count: bool,

    #[darling(default)]
    pub(crate) sort: Option<String>,

    #[darling(default)]
    pub(crate) group: bool,

    #[darling(default)]
    pub(crate) having: bool,
}

impl Field {
    pub fn validate(&self) -> Result<(), crate::Error> {
        if ![
            self.select,
            self.filter,
            self.offset,
            self.limit,
            self.count,
            self.sort.is_some(),
            self.group,
            self.having,
        ]
        .iter()
        .any(|x| *x)
        {
            return Err(crate::Error::InvalidColumn);
        }
        Ok(())
    }
}
