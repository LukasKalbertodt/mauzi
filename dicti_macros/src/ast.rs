use proc_macro2::{Literal, Span, Term, TokenNode, TokenStream, TokenTree, TokenTreeIter};


#[derive(Debug, Clone)]
pub struct Dict {
    pub trans_units: Vec<TransUnit>,
}

#[derive(Debug, Clone)]
pub struct TransUnit {
    pub name: Term,
    pub params: Vec<UnitParam>,
    pub body: UnitBody,
}

#[derive(Debug, Clone)]
pub struct UnitParam {
    pub name: Term,
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
    Lang(Term),
    // WithRegion {
    //     lang: Term,
    //     region: Term,
    // },
}


#[derive(Debug, Clone)]
pub enum ArmBody {
    Str(Literal),
    // Raw(TokenTree),
}
