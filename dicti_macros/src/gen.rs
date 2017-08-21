use proc_macro::{quote, Literal, TokenNode, TokenStream};

use dicti_runtime::Locale;

use Result;
use ast;



pub fn gen(dict: ast::Dict) -> Result<TokenStream> {
    let dict_ident = ast::Ident::export("Dict");
    let new_ident = ast::Ident::export("new");

    let methods: TokenStream = dict.trans_units.into_iter()
        .map(gen_trans_unit)
        .collect::<Result<_>>()?;

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

fn gen_trans_unit(unit: ast::TransUnit) -> Result<TokenStream> {
    let name = unit.name.exported();
    let params: TokenStream = unit.params.iter()
        .map(|param| {
            let name = param.name.exported();
            let ty = param.ty.0.parse::<TokenStream>().unwrap();

            quote! {
                , $name: $ty
            }
        })
        .collect();

    let mut has_wildcard = false;
    let last_id = unit.body.arms.len() - 1;
    let match_arms: TokenStream = unit.body.arms.into_iter()
        .enumerate()
        .map(|(i, arm)| {
            has_wildcard |= arm.pattern.is_underscore();
            let pattern = gen_arm_pattern(arm.pattern, i == last_id)?;

            let body = gen_arm_body(arm.body)?;

            Ok(quote! {
                $pattern => { $body }
            })
        })
        .collect::<Result<_>>()?;

    let wildcard_arm = if has_wildcard {
        quote! {}
    } else {
        quote! {
            _ => panic!("missing translation"),
        }
    };

    Ok(quote! {
        pub fn $name(&self $params) -> String {
            match self.locale {
                $match_arms
                $wildcard_arm
            }
        }
    })
}

fn gen_arm_pattern(pattern: ast::ArmPattern, last: bool) -> Result<TokenStream> {
    let out = match pattern {
        ast::ArmPattern::Underscore => {
            quote! { _ }
        }
        ast::ArmPattern::Lang(lang) => {
            if Locale::from_variant_str(&lang).is_some() {
                // It is referring to a variant of the `Locale` enum
                quote! { dicti::Locale::$lang(_) }
            } else {
                // It is a name for a variable binding
                quote! { $lang }
            }
        }
        ast::ArmPattern::WithRegion { lang, region } => {
            let locale = match Locale::from_variant_str(&lang) {
                None => {
                    return Err(format!(
                        "{} is not a valid language!",
                        lang.as_str(),
                    ));
                }
                Some(l) => l,
            };

            let ts = if locale.with_region_variant_str(&region).is_some() {
                let region_ty = ast::Ident::new(locale.region_type_str());
                quote! { dicti::Locale::$lang(dicti::$region_ty::$region) }
            } else {
                quote! { dicti::Locale::$lang($region) }
            };

            ts
        }
    };

    if last && !pattern.is_underscore() {
        Ok(quote! { $out if true })
    } else {
        Ok(out)
    }
}

fn gen_arm_body(body: ast::ArmBody) -> Result<TokenStream> {
    match body {
        ast::ArmBody::Str(s) => {
            let lit = TokenNode::Literal(Literal::string(&s));
            Ok(quote! { $lit.to_string() })
        }
        ast::ArmBody::Raw(ts) => Ok(ts),
    }
}
