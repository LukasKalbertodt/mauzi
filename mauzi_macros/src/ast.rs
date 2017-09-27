//! This module defines all types that make up the AST.
//!
//! The way the macro works is not very different from how many compilers work,
//! though a lot simpler:
//!
//! ```
//! ┌─────────────┐     ┌───────┐     ┌─────────────┐
//! │ TokenStream │  ➡  │  AST  │  ➡  │ TokenStream │
//! └─────────────┘     └───────┘     └─────────────┘
//!                  ↑             ↑
//!                parse          gen
//! ```
//!
//! So the AST is the abstract representation of the dictionary after parsing.
//!


use std::fmt;
use std::ops::Deref;
use proc_macro::{Span, Term, TokenNode, TokenStream, TokenTree};

use util::Spanned;


/// A dictionary, consisting of zero or more *translation units*.
#[derive(Debug, Clone)]
pub struct Dict {
    pub locale_def: LocaleDef,
    pub modules: Vec<Mod>,
    pub trans_units: Vec<TransUnit>,
}

impl Dict {
    pub fn units(&self) -> UnitsIter {
        UnitsIter {
            units: &self.trans_units,
            modules: self.modules.iter().collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mod {
    pub name: Ident,
    pub modules: Vec<Mod>,
    pub trans_units: Vec<TransUnit>,
}

/// Defines all languages and regions used by the dictionary.
#[derive(Debug, Clone)]
pub struct LocaleDef {
    pub langs: Vec<LocaleLang>,
}

impl LocaleDef {
    /// Returns the name of the `Locale` enum.
    pub fn name(&self) -> Ident {
        Ident::exported("Locale")
    }

    /// Returns the language with the given name if it exists.
    pub fn get_lang(&self, lang_name: &str) -> Option<&LocaleLang> {
        self.langs.iter()
            .find(|lang| lang.name.as_str() == lang_name)
    }
}

/// A language with an optional list of regions.
#[derive(Debug, Clone)]
pub struct LocaleLang {
    pub name: Ident,
    pub regions: Vec<Ident>,
}

impl LocaleLang {
    pub fn has_regions(&self) -> bool {
        !self.regions.is_empty()
    }

    pub fn contains_region(&self, region_name: &str) -> bool {
        self.regions.iter()
            .find(|region| region.as_str() == region_name)
            .is_some()
    }
}

/// A named translation unit, consisting of a definition and optional
/// parameters.
///
/// The name of a translation unit is commonly called *translation key*. The
/// definition of a translation unit contains definitions for different
/// languages. In those definitions, the translation unit's parameters may be
/// used.
#[derive(Debug, Clone)]
pub struct TransUnit {
    pub name: Ident,
    pub params: Option<Vec<UnitParam>>,
    pub return_type: Option<Ty>,
    pub body: UnitBody,
}

/// A paramter of a translation unit.
///
/// # Example
///
/// ```
/// name: &str
/// ```
#[derive(Debug, Clone)]
pub struct UnitParam {
    pub name: Ident,
    pub ty: Ty,
}

/// The body of a translation unit, consisting of zero or more arms.
///
/// # Example
///
/// ```
/// En(Gb) => "Hello sir",
/// De => "Guten Tag",
/// ```
#[derive(Debug, Clone)]
pub struct UnitBody {
    pub arms: Vec<UnitArm>,
}

/// One arm of a translation unit's body, consisting of a pattern/matcher and
/// a body.
///
/// # Example
///
/// ```
/// En(Gb) => "Hello sir"
/// ```
#[derive(Debug, Clone)]
pub struct UnitArm {
    pub pattern: ArmPattern,
    pub body: Spanned<ArmBody>,
}

/// One arm's pattern.
///
/// # Example
///
/// Underscore/wildcard pattern:
/// ```
/// _
/// ```
///
/// Only language given:
/// ```
/// En
/// // ... or ...
/// lang
/// ```
///
/// Language and region given
/// ```
/// En(Gb)
/// // ... or ...
/// En(region)
/// ```
#[derive(Debug, Clone)]
pub enum ArmPattern {
    Underscore(Span),
    Lang(Ident),
    WithRegion {
        lang: Ident,
        region: Ident,
    },
}

impl ArmPattern {
    /// Assumes all idents used in this pattern have spans.
    #[allow(dead_code)]
    pub fn span(&self) -> Span {
        match *self {
            ArmPattern::Underscore(span) => span,
            ArmPattern::Lang(lang) => lang.span().unwrap(),

            // TODO: join these two spans!
            ArmPattern::WithRegion { lang, .. } => lang.span().unwrap(),
        }
    }
}

impl fmt::Display for ArmPattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArmPattern::Underscore(_) => "_".fmt(f),
            ArmPattern::Lang(lang) => lang.fmt(f),
            ArmPattern::WithRegion { lang, region } => {
                write!(f, "{}({})", lang, region)
            }
        }
    }
}

/// The body of one arm.
///
/// Right now we support two kinds of bodies:
/// - String literals with placeholders
/// - Raw Rust code
///
/// # Example
///
/// String literal:
/// ```
/// "Hallo {user}"
/// ```
///
/// Raw Rust body:
/// ```
/// {
///     format!("Hallo {}", user)
/// }
/// ```
#[derive(Debug, Clone)]
pub enum ArmBody {
    Str(String),
    Raw(TokenStream),
}

impl ArmBody {
    pub fn is_raw_block(&self) -> bool {
        match *self {
            ArmBody::Raw(_) => true,
            _ => false,
        }
    }
}

/// A Rust type.
///
/// Since we don't want to replicate the Rust type parsing algorithm, we are
/// lazy and simply store all tokens as a string.
#[derive(Debug, Clone)]
pub struct Ty(pub String);

/// An identificator (some word like thing).
#[derive(Debug, Clone, Copy)]
pub struct Ident {
    term: Term,
    span: Option<Span>,
}

impl Ident {
    /// Creates a new ident from the given term and span.
    pub fn new(term: Term, span: Span) -> Self {
        Self {
            term,
            span: Some(span),
        }
    }

    /// Creates a `Ident` which won't be visible to the calling code. It won't
    /// have a span, so you can't emit a spanned diagnostic for this ident.
    ///
    /// This function should only be used for new, generated idents.
    pub fn internal(s: &str) -> Self {
        Self {
            term: Term::intern(s),
            span: None,
        }
    }

    /// Returns an ident in the same expansion context as the macro invocation.
    /// This means that the user can refer to this ident.
    ///
    /// This function should only be used for new, generated idents.
    pub fn exported(s: &str) -> Self {
        Self {
            term: Term::intern(s),
            span: Some(Span::call_site()),
        }
    }

    pub fn as_str(&self) -> &str {
        self.term.as_str()
    }

    pub fn span(&self) -> Option<Span> {
        self.span
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Deref for Ident {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Into<TokenStream> for Ident {
    /// This implementation returns a token stream that represents this ident
    /// in a macro internal expansion context. Thus the user cannot access this
    /// name.
    fn into(self) -> TokenStream {

        match self.span {
            Some(span) => {
                // If we have a span, we use it.
                TokenTree {
                    span: span,
                    kind: TokenNode::Term(self.term),
                }.into()
            }
            None => {
                // Otherwise we don't use a span by converting a TokenNode into
                // a TokenStream directly.
                TokenNode::Term(self.term).into()
            }
        }
    }
}



// ===========================================================================
// Helper
// ===========================================================================

pub struct UnitsIter<'a> {
    units: &'a [TransUnit],
    modules: Vec<&'a Mod>,
}

impl<'a> Iterator for UnitsIter<'a> {
    type Item = &'a TransUnit;
    fn next(&mut self) -> Option<Self::Item> {
        // If we have no immediate units that we can yield, we have to get new
        // ones from our modules.
        if self.units.is_empty() {
            // We resolve the next modules until we found some translation
            // units.
            while let Some(m) = self.modules.pop() {
                self.units = &m.trans_units;
                self.modules.extend(&m.modules);

                if !self.units.is_empty() {
                    break;
                }
            }
        }

        // If we don't have any translation units at this point, we have
        // already resolved all modules without finding translation units. So
        // we are exhausted.
        if self.units.is_empty() {
            None
        } else {
            let out = &self.units[0];
            self.units = &self.units[1..];
            Some(out)
        }
    }
}
