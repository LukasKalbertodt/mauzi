use std::ops::Deref;
use proc_macro::{Span, Term, TokenNode, TokenStream, TokenTree};


#[derive(Debug, Clone)]
pub struct Dict {
    // pub header: DictHeader,
    pub trans_units: Vec<TransUnit>,
}

#[derive(Debug, Clone)]
pub struct DictHeader {
    pub dict_name: Ident,
}

#[derive(Debug, Clone)]
pub struct TransUnit {
    pub name: Ident,
    pub params: Vec<UnitParam>,
    pub body: UnitBody,
}

#[derive(Debug, Clone)]
pub struct UnitParam {
    pub name: Ident,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub struct Ty(pub String);

#[derive(Debug, Clone)]
pub struct UnitBody {
    pub arms: Vec<UnitArm>,
}

#[derive(Debug, Clone)]
pub struct UnitArm {
    pub pattern: ArmPattern,
    pub body: ArmBody,
}

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

#[derive(Debug, Clone)]
pub enum ArmBody {
    Str(String),
    Raw(TokenStream),
}

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

    pub fn exported(&self) -> TokenTree {
        TokenTree {
            span: Span::call_site(),
            kind: TokenNode::Term(self.term),
        }
    }

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
    fn into(self) -> TokenStream {
        TokenNode::Term(self.term).into()
    }
}
