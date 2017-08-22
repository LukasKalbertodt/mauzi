use std::iter::Peekable;

use proc_macro::{Delimiter, Literal, Spacing, TokenNode, TokenStream, TokenTree, TokenTreeIter};
use literalext::LiteralExt;

use ast;
use Result;


/// Parses the input token stream into an abstract intermediate representation.
pub fn parse(input: TokenStream) -> Result<ast::Dict> {
    let mut iter = Iter::new(input);

    // Collect all translation units in this dictonary.
    let mut trans_units = Vec::new();
    while !iter.is_exhausted() {
        trans_units.push(parse_trans_unit(&mut iter)?);
    }

    Ok(ast::Dict { trans_units })
}

/// Parses one translation unit from the given iterator.
fn parse_trans_unit(iter: &mut Iter) -> Result<ast::TransUnit> {
    // Each translation unit starts with a name (required).
    let name = iter.eat_term()?;

    // Get the parsed parameters and the group (brace delimited block)
    // representing the body.
    let (params, body_group) = {
        // The translation unit's name needs to be followed by a group. Two
        // kinds of groups are valid: brace-delimited (body) and parenthesis
        // delimited (parameters).
        let (delim, group) = iter.eat_group()?;
        if eq_delim(delim, Delimiter::Parenthesis) {
            // The translation unit has parameters. It might still have 0
            // parameters, but there is at least the `()` pair for parameters.
            // Parse this group now.
            let params = parse_unit_params(group)?;

            // Get the next group which is hopefully a valid body
            let body = iter.eat_group_delimited_by(Delimiter::Brace)?;

            (params, body)
        } else if eq_delim(delim, Delimiter::Brace) {
            // This is already the group representing the body! This
            // translation unit doesn't have any parameters.
            (vec![], group)
        } else {
            // Syntax error!
            let msg = format!(
                "expected block starting with '{{' or '(', found block starting with {:?}",
                delim
            );
            return Err(msg);
        }
    };

    // Parse the group representing the body.
    let body = parse_unit_body(body_group)?;

    Ok(ast::TransUnit {
        name,
        params,
        body,
    })
}

/// Parse the given group as parameters of a translation unit.
fn parse_unit_params(group: TokenStream) -> Result<Vec<ast::UnitParam>> {
    let mut iter = Iter::new(group);

    // Collect all parameters.
    let mut params = Vec::new();
    while !iter.is_exhausted() {
        // A parameter needs a name ...
        let name = iter.eat_term()?;
        // ... followed by a colon ...
        iter.eat_op_if(':')?;
        // ... followed by a type.
        let ty = parse_type(&mut iter)?;

        params.push(ast::UnitParam { name, ty });

        // Eat one comma, if haven't reached the end.
        if !iter.is_exhausted() {
            iter.eat_op_if(',')?;
        }
    }

    Ok(params)
}

/// Parses a Rust type from the given iterator.
///
/// Note that this is actually not really parsing a Rust type. It simply adds
/// all potentially valid tokens (all except `,`) to a string buffer.
/// Duplicating the Rust type parsing algorithm would be overkill. Thus we
/// won't detect syntax errors at this stage.
fn parse_type(iter: &mut Iter) -> Result<ast::Ty> {
    use std::fmt::Write;

    let mut ty = String::new();
    loop {
        // We want to stop when we reached the end of the iterator or when we
        // reach a comma. However, we don't want to consume the comma.
        match iter.peek_curr() {
            Err(_) => break,
            Ok(&TokenTree { kind: TokenNode::Op(op, _), .. }) if op == ',' => break,
            _ => {},
        }

        // Apparantly we didn't stop, so we will add this token to our string.
        write!(ty, "{}", iter.eat_curr().unwrap()).unwrap();
    }

    Ok(ast::Ty(ty))
}

/// Parses a translation unit's body from the given group.
fn parse_unit_body(group: TokenStream) -> Result<ast::UnitBody> {
    let mut iter = Iter::new(group);

    // Collect all arms.
    let mut arms = Vec::new();
    while !iter.is_exhausted() {
        // Each arm starts with a pattern/matcher ...
        let pattern = parse_arm_pattern(&mut iter)?;

        // ... followed by a `=>` ...
        if eq_spacing(iter.eat_op_if('=')?, Spacing::Alone) {
            return Err("expected '=>', found '='".into());
        }
        iter.eat_op_if('>')?;

        // ... followed by the actual body.
        let body = parse_arm_body(&mut iter)?;

        // Maybe eat comma, if haven't reached the end
        if !iter.is_exhausted() {
            if body.is_raw_block() {
                // If the last body was a raw block (delimited by braces) it's
                // ok to not have a comma.
                let _ = iter.eat_op_if(',');
            } else {
                // If the body was not a raw block, we need a comma!
                iter.eat_op_if(',')?;
            }
        }

        arms.push(ast::UnitArm {
            pattern, body
        });
    }

    Ok(ast::UnitBody { arms })
}

/// Parses one arm's pattern from the given iterator.
fn parse_arm_pattern(iter: &mut Iter) -> Result<ast::ArmPattern> {
    if iter.eat_op_if('_').is_ok() {
        // The pattern is a wildcard pattern.
        Ok(ast::ArmPattern::Underscore)
    } else {
        // The pattern has at least the language component which starts with
        // a term.
        let lang = iter.eat_term()?;

        // Next, there could be a group to specify a region.
        if !iter.peek_curr()?.kind.is_group() {
            // Simple case: only a language is given.
            Ok(ast::ArmPattern::Lang(lang))
        } else {
            // More complex case: language and region are given.
            let region_group = iter.eat_group_delimited_by(Delimiter::Parenthesis)?;

            // Inside the group we expect only one term and nothing more
            let mut inner_iter = Iter::new(region_group);
            let region = inner_iter.eat_term()?;
            if let Ok(tok) = inner_iter.eat_curr() {
                return Err(format!("didn't expect token '{:?}' in matcher", tok));
            }

            Ok(ast::ArmPattern::WithRegion {
                lang,
                region,
            })
        }
    }
}

/// Parses the body of one arm.
fn parse_arm_body(iter: &mut Iter) -> Result<ast::ArmBody> {
    // If we encounter a group next, we know the body is raw Rust.
    if iter.peek_curr()?.kind.is_group() {
        // Raw Rust body
        let group = iter.eat_group_delimited_by(Delimiter::Brace)?;
        Ok(ast::ArmBody::Raw(group))
    } else {
        // A standard body consisting of a single literal.
        let lit = iter.eat_literal()?;
        match lit.parse_string() {
            Some(s) => Ok(ast::ArmBody::Str(s)),
            None => Err(format!("expected string literal, found '{}'", lit)),
        }
    }
}

/// A helper type wrapping an iterator over token-trees. Has many helper
/// methods for retreiving specific token kinds from the iterator.
struct Iter(Peekable<TokenTreeIter>);

impl Iter {
    fn new(input: TokenStream) -> Self {
        Iter(input.into_iter().peekable())
    }

    /// Consumes and returns the next tt if it is a `Term`. Otherwise an `Err`
    /// is returned.
    fn eat_term(&mut self) -> Result<ast::Ident> {
        match self.eat_curr()? {
            TokenTree { kind: TokenNode::Term(term), .. } => Ok(term.into()),
            other => return Err(format!("expected `term`, found '{}'", other)),
        }
    }

    /// Consumes and returns the next tt if it is a `Group`. Otherwise an `Err`
    /// is returned.
    fn eat_group(&mut self) -> Result<(Delimiter, TokenStream)> {
        match self.eat_curr()? {
            TokenTree { kind: TokenNode::Group(delim, ts), .. } => Ok((delim, ts)),
            other => return Err(format!("expected `group`, found '{}'", other)),
        }
    }

    /// Consumes and returns the next tt if it is a `Group` delimited by the
    /// given `delim`. Otherwise an `Err` is returned.
    fn eat_group_delimited_by(&mut self, delim: Delimiter) -> Result<TokenStream> {
        let (actual_delim, group) = self.eat_group()?;
        if eq_delim(delim, actual_delim) {
            Ok(group)
        } else {
            let msg = format!(
                "expected block starting with '{:?}', found block starting with '{:?}'",
                delim,
                actual_delim,
            );
            Err(msg)
        }
    }

    /// Consumes and returns the next tt if it is a `Literal`. Otherwise an
    /// `Err` is returned.
    fn eat_literal(&mut self) -> Result<Literal> {
        match self.eat_curr()? {
            TokenTree { kind: TokenNode::Literal(lit), .. } => Ok(lit),
            other => return Err(format!("expected `group`, found '{}'", other)),
        }
    }

    /// Consumes and returns the next tt if it equals the given operator.
    /// Otherwise an `Err` is returned.
    fn eat_op_if(&mut self, op: char) -> Result<Spacing> {
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
    fn peek_curr(&mut self) -> Result<&TokenTree> {
        self.0.peek().ok_or("unexpected EOF".into())
    }

    /// Consumes and returns the current tt or an error if there is no next tt.
    fn eat_curr(&mut self) -> Result<TokenTree> {
        self.0.next().ok_or("unexpected EOF".into())
    }

    /// Returns `true` if the iterator is exhausted and won't generate new
    /// token trees anymore.
    fn is_exhausted(&mut self) -> bool {
        self.0.peek().is_none()
    }

    /// Advances the iterator once, thus consuming the current tt.
    fn bump(&mut self) {
        self.0.next();
    }

    /// Returns a debug representation of all token trees inside this iterator.
    #[allow(dead_code)]
    fn into_debug_output(self) -> String {
        self.0.map(|e| format!("{:?}\n", e)).collect()
    }
}

/// Helper extension trait to check the kind of a `TokenNode`.
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

/// Compares two `Delimiter`
fn eq_delim(a: Delimiter, b: Delimiter) -> bool {
    use proc_macro::Delimiter::*;
    match a {
        Parenthesis => if let Parenthesis = b { true } else { false },
        Brace => if let Brace = b { true } else { false },
        Bracket => if let Bracket = b { true } else { false },
        None => if let None = b { true } else { false },
    }
}

/// Compares two `Spacing`
fn eq_spacing(a: Spacing, b: Spacing) -> bool {
    use proc_macro::Spacing::*;
    match a {
        Alone => if let Alone = b { true } else { false },
        Joint => if let Joint = b { true } else { false },
    }
}
