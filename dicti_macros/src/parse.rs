use std::iter::Peekable;

use proc_macro2::{Delimiter, Literal, Spacing, Span, Term, TokenNode, TokenStream, TokenTree, TokenTreeIter};

use ast;
use ParseResult;



pub fn parse(input: TokenStream) -> ParseResult<ast::Dict> {
    let mut iter = Iter::new(input);
    let mut trans_units = Vec::new();

    while iter.is_not_exhausted() {
        trans_units.push(parse_trans_unit(&mut iter)?);
    }

    let ast_root = ast::Dict {
        trans_units
    };

    Ok(ast_root)
}

fn parse_trans_unit(iter: &mut Iter) -> ParseResult<ast::TransUnit> {
    let name = iter.eat_term()?;

    let (params, body_group) = {
        let (delim, group) = iter.eat_group()?;
        if delim == Delimiter::Parenthesis {
            // The translation unit has parameters. It might still have 0
            // parameters, but there is at least the `()` pair for
            // parameters.
            let params = parse_unit_params(group)?;


            // Get the next group which is hopefully a valid body
            let (delim, body) = iter.eat_group()?;
            if delim != Delimiter::Brace {
                return Err(format!("expected group starting with '{{', found {:?}", delim));
            }

            (params, body)
        } else if delim == Delimiter::Brace {
            // This is already the group representing the body! This
            // translation unit doesn't have any parameters.
            (vec![], group)
        } else {
            // Syntax error!
            let msg = format!("expected group starting with '{{' or '(', found {:?}", delim);
            return Err(msg);
        }
    };

    let body = parse_unit_body(body_group)?;

    Ok(ast::TransUnit {
        name,
        params,
        body,
    })
}

fn parse_unit_params(group: TokenStream) -> ParseResult<Vec<ast::UnitParam>> {
    let mut iter = Iter::new(group);
    let mut params = Vec::new();
    while iter.is_not_exhausted() {
        let name = iter.eat_term()?;
        let (op, _) = iter.eat_op()?;
        if op != ':' {
            return Err(format!("expected ':', found '{}'", op));
        }

        let ty = parse_type(&mut iter)?;
        params.push(ast::UnitParam { name, ty });

        // Eat comma, if haven't reached the end
        if iter.is_not_exhausted() {
            iter.eat_op_if(',')?;
        }
    }

    Ok(params)
}

fn parse_type(iter: &mut Iter) -> ParseResult<ast::Ty> {
    use std::fmt::Write;

    let mut ty = String::new();
    loop {
        match iter.peek_curr() {
            Err(_) => break,
            Ok(&TokenTree { kind: TokenNode::Op(op, _), .. }) if op == ',' => break,
            _ => {},
        }
        write!(ty, "{}", iter.eat_curr().unwrap()).unwrap();
    }

    Ok(ast::Ty(ty))
}

fn parse_unit_body(group: TokenStream) -> ParseResult<ast::UnitBody> {
    let mut iter = Iter::new(group);
    let mut arms = Vec::new();

    while iter.is_not_exhausted() {
        let pattern = parse_arm_pattern(&mut iter)?;

        // Next, we need a `=>`
        if let Spacing::Alone = iter.eat_op_if('=')? {
            return Err("expected '=>', found '='".into());
        }
        iter.eat_op_if('>')?;

        let string = iter.eat_literal()?;

        let body = ast::ArmBody::Str(string);

        arms.push(ast::UnitArm {
            pattern, body
        });

        // Eat comma, if haven't reached the end
        if iter.is_not_exhausted() {
            iter.eat_op_if(',')?;
        }
    }

    Ok(ast::UnitBody { arms })
}

fn parse_arm_pattern(iter: &mut Iter) -> ParseResult<ast::ArmPattern> {
    if iter.eat_op_if('_').is_ok() {
        Ok(ast::ArmPattern::Underscore)
    } else {
        let top_level = iter.eat_term()?;
        // TODO: second level

        Ok(ast::ArmPattern::Lang(top_level))
    }
}

struct Iter(Peekable<TokenTreeIter>);

impl Iter {
    fn new(input: TokenStream) -> Self {
        Iter(input.into_iter().peekable())
    }

    /// Consumes and returns the next tt if it is a `Term`. Otherwise an `Err`
    /// is returned.
    fn eat_term(&mut self) -> ParseResult<Term> {
        match self.eat_curr()? {
            TokenTree { kind: TokenNode::Term(term), .. } => Ok(term),
            other => return Err(format!("expected `term`, found '{}'", other)),
        }
    }

    fn eat_group(&mut self) -> ParseResult<(Delimiter, TokenStream)> {
        match self.eat_curr()? {
            TokenTree { kind: TokenNode::Group(delim, ts), .. } => Ok((delim, ts)),
            other => return Err(format!("expected `group`, found '{}'", other)),
        }
    }

    fn eat_op(&mut self) -> ParseResult<(char, Spacing)> {
        match self.eat_curr()? {
            TokenTree { kind: TokenNode::Op(op, s), .. } => Ok((op, s)),
            other => return Err(format!("expected `group`, found '{}'", other)),
        }
    }

    fn eat_literal(&mut self) -> ParseResult<Literal> {
        match self.eat_curr()? {
            TokenTree { kind: TokenNode::Literal(lit), .. } => Ok(lit),
            other => return Err(format!("expected `group`, found '{}'", other)),
        }
    }

    fn eat_op_if(&mut self, op: char) -> ParseResult<Spacing> {
        let out = match *self.peek_curr()? {
            TokenTree { kind: TokenNode::Op(found_op, spacing), .. } => {
                if found_op == op {
                    spacing
                } else {
                    let msg = format!("expected '{}', found '{}'", op, found_op);
                    return Err(msg)
                }
            }
            ref other => return Err(format!("expected '{}', found '{}'", op, other)),
        };
        self.bump();
        Ok(out)
    }


    /// Peeks onto the current tt. Returns an error if there is no next tt.
    fn peek_curr(&mut self) -> ParseResult<&TokenTree> {
        self.0.peek().ok_or("unexpected EOF".into())
    }

    /// Returns the current tt or an error if there is no next tt.
    fn eat_curr(&mut self) -> ParseResult<TokenTree> {
        self.0.next().ok_or("unexpected EOF".into())
    }

    fn is_not_exhausted(&mut self) -> bool {
        self.0.peek().is_some()
    }

    /// Advances the iterator once, thus consuming the current tt.
    fn bump(&mut self) {
        self.0.next();
    }

    fn into_debug_output(self) -> String {
        self.0.map(|e| format!("{:?}\n", e)).collect()
    }
}

trait TokenNodeExt {
    fn is_group(&self) -> bool;
    fn is_term(&self) -> bool;
    fn is_op(&self) -> bool;
    fn is_literal(&self) -> bool;
}

impl TokenNodeExt for TokenNode {
    fn is_group(&self) -> bool {
        match *self {
            TokenNode::Group(..) => true,
            _ => false,
        }
    }
    fn is_term(&self) -> bool {
        match *self {
            TokenNode::Term(..) => true,
            _ => false,
        }
    }
    fn is_op(&self) -> bool {
        match *self {
            TokenNode::Op(..) => true,
            _ => false,
        }
    }
    fn is_literal(&self) -> bool {
        match *self {
            TokenNode::Literal(..) => true,
            _ => false,
        }
    }
}
