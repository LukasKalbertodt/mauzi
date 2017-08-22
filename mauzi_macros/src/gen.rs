use proc_macro::{quote, Literal, TokenNode, TokenStream};

use mauzi_runtime::Locale;

use Result;
use ast;


/// Generates the resulting Rust code from the AST.
///
/// This function is the "compiler backend" of this proc macro: it takes all
/// the intermediate representations (in this case, this is simply the AST) and
/// produces an output.
///
/// Specifically, for each `dict!{}` invocation, a new struct-type called
/// `Dict` is generated. This type has a `new()` function to create an instance
/// of it from a `Locale`.
///
/// For each translation key, the type has one method with the name and the
/// parameters of said key. This method internally matches over the actual
/// locale to decide which "body" to use. Those methods always return a
/// `String`.
pub fn gen(dict: ast::Dict) -> Result<TokenStream> {
    // We want to create a few new names which the user can refer to. Due to
    // macro hygiene, we have to create special ident-tokens that live in the
    // same "context" as the invocation of `dict!{}` is in. Otherwise, the
    // names would be hidden/trapped inside of our macro context.
    let dict_ident = ast::Ident::export("Dict");
    let new_ident = ast::Ident::export("new");

    // We generate the token streams for all methods and combine them into a
    // big token stream.
    let methods = dict.trans_units.into_iter()
        .map(gen_trans_unit)
        .collect::<Result<TokenStream>>()?;


    // Now we just return this quoted Rust code.
    //
    // We need to refer to the `Locale` type from the `mauzi_runtime` crate,
    // but there isn't a good way to do that currently.
    Ok(quote! {
        extern crate mauzi;

        pub struct $dict_ident {
            locale: mauzi::Locale,
        }

        impl $dict_ident {
            pub fn $new_ident(locale: mauzi::Locale) -> Self {
                Self { locale }
            }

            $methods
        }
    })
}

/// Takes one translation unit and generates the corresponding Rust code.
fn gen_trans_unit(unit: ast::TransUnit) -> Result<TokenStream> {
    // ===== Function signature ==============================================
    // We want to make the name of the translation unit available to the user.
    let name = unit.name.exported();

    // Generate code for all parameters, merging all together into one
    // token stream.
    let params: TokenStream = unit.params.iter().map(|param| {
        // We also need to make the name of the parameter available to the
        // user, because the raw body provided by the user uses those
        // parameters and those indents are in the user's expansion
        // context.
        let name = param.name.exported();

        // We store the type as a simple `String` in the AST so we need to
        // parse it to a token stream. We know that it can be parsed
        // correctly, since we create the string from a token stream.
        let ty = param.ty.0.parse::<TokenStream>().unwrap();

        quote! {
            , $name: $ty
        }
    }).collect();

    // ===== Function body ===================================================
    // Find out if the user already provided a wildcard arm. If not, we'll
    // generate one later.
    let mut has_wildcard = false;

    let last_id = unit.body.arms.len() - 1;

    // Generate a match arm for each translation arm.
    let match_arms: TokenStream = unit.body.arms.into_iter()
        .enumerate()
        .map(|(i, arm)| {
            has_wildcard |= arm.pattern.is_underscore();

            // Generate the *matcher* (the left part of a match arm).
            let pattern = gen_arm_pattern(arm.pattern, i == last_id)?;

            // Generate the body of the match arm.
            let body = gen_arm_body(arm.body)?;

            // Combine both into the full match arm
            Ok(quote! {
                $pattern => { $body }
            })
        })
        .collect::<Result<_>>()?;

    // If the user didn't provide a wildcard arm, we need to add one.
    let wildcard_arm = if has_wildcard {
        quote! {}
    } else {
        // TODO: maybe we don't want to panic here! Best idea would be to let
        // the user decide.
        quote! {
            _ => panic!("missing translation"),
        }
    };

    // Combine everything into the method.
    Ok(quote! {
        pub fn $name(&self $params) -> String {
            match self.locale {
                $match_arms
                $wildcard_arm
            }
        }
    })
}

/// Generates the *matcher* (the left side) of a match arm.
fn gen_arm_pattern(pattern: ast::ArmPattern, last: bool) -> Result<TokenStream> {
    let out = match pattern {
        ast::ArmPattern::Underscore => {
            quote! { _ }
        }

        // The user only matches on the language and doesn't care about the
        // region.
        ast::ArmPattern::Lang(lang) => {
            // We need to decide whether the user provided a constant language
            // to match against or a variable name to bind the language to. We
            // find out by trying to find a language with the given name. If
            // there doesn't exist one, we assume it's meant as a variable
            // binding.
            if Locale::from_variant_str(&lang).is_some() {
                // It is referring to a variant of the `Locale` enum
                quote! { mauzi::Locale::$lang(_) }
            } else {
                // It is a name for a variable binding
                quote! { $lang }
            }
        }

        // The user matches against language and region (or at least wants to
        // bind the region to a variable).
        ast::ArmPattern::WithRegion { lang, region } => {
            // This time, the language has to be a variant of the `Locale`
            // enum. If not we're gonna emit an error.
            let locale = match Locale::from_variant_str(&lang) {
                None => {
                    return Err(format!(
                        "{} is not a valid language!",
                        lang.as_str(),
                    ));
                }
                Some(l) => l,
            };

            // Next we need to again figure out whether the user provided a
            // region constant or a variable name to bind to.
            if locale.with_region_variant_str(&region).is_some() {
                // Constant region to match against...
                let region_ty = ast::Ident::new(locale.region_type_str());
                quote! { mauzi::Locale::$lang(mauzi::$region_ty::$region) }
            } else {
                // Variable to bind to
                quote! { mauzi::Locale::$lang($region) }
            }
        }
    };

    // Here we need to perform a special trick. The problem is that we need to
    // provide a wildcard arm to the match block in order to make the code
    // compile, even when the user didn't make the match exhaustive. We cannot
    // easily check whether or not the users options exhaust the match, so in
    // some cases we'll add a wildcard arm although it cannot be reached. This
    // emits a compiler warning.
    //
    // We could disable the warning, but we actually want the warning for the
    // user's code. The idea is to inject a `if true` match guard to one match
    // arm given by the user. The compiler doesn't inspect match guards, so it
    // won't be able to tell that the match is already exhaustive.
    //
    // This is a hack, but it's fine for now.
    if last && !pattern.is_underscore() {
        Ok(quote! { $out if true })
    } else {
        Ok(out)
    }
}

/// Generates the body of a match arm.
fn gen_arm_body(body: ast::ArmBody) -> Result<TokenStream> {
    match body {
        ast::ArmBody::Raw(ts) => Ok(ts),
        ast::ArmBody::Str(s) => {
            // We need to convert the fancy placeholder string into a
            // `format!()` expression. We do this by first going through the
            // fancy format string with an FSA like algorithm, splitting it
            // into the real format string and the arguments.

            #[derive(Clone, Copy)]
            enum State {
                /// The last char we read belonged to the real format string
                /// and will be printed verbatim, or (special case) we just
                /// exited a placeholder.
                Normal,
                /// The last char we read was part of a placeholder, or
                /// (special case) we just entered a placeholder.
                InPlaceholder,
            }

            let mut state = State::Normal;
            let mut it = s.chars().peekable();

            // We will pass `format_str` as the first argument of `format!()`
            // later. `args` contains all other arguments.
            let mut format_str = String::new();
            let mut args = Vec::new();

            while let Some(c) = it.next() {
                match (state, c) {
                    // Entering a placeholder
                    (State::Normal, '{') => {
                        // If the next one is `{` it's an escaped brace and we
                        // shall copy both braces verbatim to the format
                        // string.
                        if let Some(&'{') = it.peek() {
                            it.next();
                            format_str.push_str("{{");
                        } else {
                            // Start a new argument and change the state.
                            args.push(String::new());
                            state = State::InPlaceholder;
                        }
                    }
                    // Outside of a placeholder, just copying
                    (State::Normal, _) => {
                        format_str.push(c);
                    }
                    // Exiting a placeholder
                    (State::InPlaceholder, '}') => {
                        format_str.push_str("{}");
                        state = State::Normal;
                    }
                    // Inside of a placeholder, copying to the last argument
                    (State::InPlaceholder, _) => {
                        args.last_mut().unwrap().push(c);
                    }
                }
            }

            // We have to parse all argument as token stream: we don't want to
            // pass them to `format!()` as string literal, but as Rust
            // expression. We concat all arguments into one token stream.
            let format_args = args.into_iter().map(|arg_s| {
                // Try to parse.
                arg_s.parse::<TokenStream>()
                    .map_err(|e| format!("not a valid Rust expression in placeholder: {:?}", e))
                    // Add a leading comma for concatting all arguments.
                    .map(|ts| quote! { , $ts })
            }).collect::<Result<TokenStream>>()?;

            // We pass the format string as a literal to `format!()`.
            let format_str = TokenNode::Literal(Literal::string(&format_str));

            Ok(quote! {
                format!($format_str $format_args)
            })
        }
    }
}
