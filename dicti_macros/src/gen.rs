use proc_macro::{quote, TokenStream};

use Result;
use ast;


/// This is a helper type to be able to use a list of something in the
/// `quote!()` macro.
#[derive(Clone, Debug)]
struct TokenStreamList(Vec<TokenStream>);

impl Into<TokenStream> for TokenStreamList {
    fn into(self) -> TokenStream {
        self.0.into_iter().collect()
    }
}

pub fn gen(dict: &ast::Dict) -> Result<TokenStream> {
    let dict_ident = ast::Ident::export("Dict");
    let new_ident = ast::Ident::export("new");

    let methods: TokenStream = dict.trans_units.iter()
        .map(gen_trans_unit)
        .collect();

    let tokens = quote! {
        extern crate dicti;

        pub struct $dict_ident {
            locale: dicti::Locale,
        }

        impl $dict_ident {
            pub fn $new_ident(locale: dicti::Locale) -> Self {
                Self { locale }
            }

            $methods
        }
    };

    Ok(tokens)
}

fn gen_trans_unit(unit: &ast::TransUnit) -> TokenStream {
    let name = unit.name.exported();
    let params: TokenStream = unit.params.iter()
        .map(|param| {
            let name = param.name.hidden();
            let ty = param.ty.0.parse::<TokenStream>().unwrap();

            quote! {
                , $name: $ty
            }
        })
        .collect();


    quote! {
        pub fn $name(&self $params) -> String {
            match self.locale {
                _ => panic!("missing translation"),
            }
        }
    }
}
