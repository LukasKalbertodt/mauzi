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


use std::ops::Deref;
use proc_macro::{Span, Term, TokenNode, TokenStream, TokenTree};


/// A dictionary, consisting of zero or more *translation units*.
#[derive(Debug, Clone)]
pub struct Dict {
    pub trans_units: Vec<TransUnit>,
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
    pub params: Vec<UnitParam>,
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
    pub body: ArmBody,
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
    Underscore,
    Lang(Ident),
    WithRegion {
        lang: Ident,
        region: Ident,
    },
}

impl ArmPattern {
    pub fn is_underscore(&self) -> bool {
        match *self {
            ArmPattern::Underscore => true,
            _ => false,
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
}

impl Ident {
    pub fn new(s: &str) -> Self {
        Self {
            term: Term::intern(s),
        }
    }

    /// Returns a token tree representing this ident in the same expansion
    /// context as the macro invocation. This means that the user can use this
    /// name.
    pub fn exported(&self) -> TokenTree {
        TokenTree {
            span: Span::call_site(),
            kind: TokenNode::Term(self.term),
        }
    }

    /// Intern the given string with `new()` and call `exported()`.
    pub fn export(s: &str) -> TokenTree {
        Self::new(s).exported()
    }

    pub fn as_str(&self) -> &str {
        self.term.as_str()
    }
}

impl Deref for Ident {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<Term> for Ident {
    fn from(term: Term) -> Self {
        Self { term }
    }
}

impl Into<TokenStream> for Ident {
    /// This implementation returns a token stream that represents this ident
    /// in a macro internal expansion context. Thus the user cannot access this
    /// name.
    fn into(self) -> TokenStream {
        TokenNode::Term(self.term).into()
    }
}
