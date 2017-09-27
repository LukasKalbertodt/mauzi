use std::io;
use std::iter::Peekable;
use std::path::Path;

use proc_macro::{
    Delimiter, Diagnostic, Level, Literal, Spacing, Span, TokenNode,
    TokenStream, TokenTree, TokenTreeIter
};
use literalext::LiteralExt;

use ast::{self, Ident};
use Result;
use util::Spanned;


/// Parses the input token stream into an abstract intermediate representation.
pub fn parse(input: TokenStream) -> Result<ast::Dict> {
    use std::env;

    // TODO: Oh boy, this is ugly. Well, we can't find out the path of the
    // `mauzi!` invocation, so we have to cheat a bit. I think this is the best
    // we can do right now: getting the manifest dir and just assuming that
    // `mauzi!` was called at the top level.
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&manifest_dir).join("src");

    let mut iter = Iter::new(input);
    let locale_def = parse_locale_def(&mut iter)?;
    let (modules, trans_units) = parse_items(&mut iter, &src_dir)?;

    Ok(ast::Dict { locale_def, modules, trans_units })
}

fn parse_locale_def(iter: &mut Iter) -> Result<ast::LocaleDef> {
    // We require `enum Locale` in the very beginning.
    iter.eat_keyword("enum")?;
    iter.eat_keyword("Locale")?;

    let body = iter.eat_group_delimited_by(Delimiter::Brace)?;
    let mut body_iter = Iter::new(body.obj);

    // Collect all langs.
    let mut langs = Vec::new();
    while !body_iter.is_exhausted() {
        langs.push(parse_locale_variant(&mut body_iter)?);

        // Maybe eat comma, if haven't reached the end
        if !body_iter.is_exhausted() {
            let _ = body_iter.eat_op_if(',');
        }
    }


    Ok(ast::LocaleDef { langs })
}

fn parse_locale_variant(iter: &mut Iter) -> Result<ast::LocaleLang> {
    let name = iter.eat_term()?;

    let mut regions = Vec::new();
    if let Ok(&TokenTree { kind: TokenNode::Group(Delimiter::Brace, _), .. }) = iter.peek_curr() {
        let body = iter.eat_group_delimited_by(Delimiter::Brace)?;
        let mut body_iter = Iter::new(body.obj);

        // Collect all regions.
        while !body_iter.is_exhausted() {
            regions.push(body_iter.eat_term()?);

            // Maybe eat comma, if haven't reached the end
            if !body_iter.is_exhausted() {
                let _ = body_iter.eat_op_if(',');
            }
        }
    }

    Ok(ast::LocaleLang {
        name,
        regions,
    })
}

fn parse_items(iter: &mut Iter, root_path: &Path) -> Result<(Vec<ast::Mod>, Vec<ast::TransUnit>)> {
    // Collect all translation units and modules.
    let mut trans_units = Vec::new();
    let mut modules = Vec::new();
    while !iter.is_exhausted() {
        let item_kind = iter.eat_term()?;
        match item_kind.as_str() {
            "unit" => trans_units.push(parse_trans_unit(iter)?),
            "mod" => modules.push(parse_module(iter, root_path)?),
            s => {
                return err!(item_kind.span().unwrap(), "expected item, found identifier '{}'", s);
            }
        }
    }

    Ok((modules, trans_units))
}

fn parse_module(iter: &mut Iter, root_path: &Path) -> Result<ast::Mod> {
    use std::fs::File;
    use std::io::Read;

    // A module declaration has the form `mod name;`. The `mod` keyword was
    // already consumed by the calling function.
    let name = iter.eat_term()?;
    let name_span = name.span().unwrap();
    iter.eat_op_if(';')?;

    // Both valid paths.
    let p0 = root_path
        .join(name.as_str())
        .join("mod.mauzi.rs");
    let p1 = root_path.join(format!("{}.mauzi.rs", name.as_str()));

    // Check that only one of those two files actually exists.
    let p = match (p0.exists(), p1.exists()) {
        (false, false) => {
            return err!(
                name_span,
                "cannot find either of those files: '{}' or '{}'",
                p0.display(),
                p1.display()
            );
        }
        (true, true) => {
            return err!(
                name_span,
                "Ambiguity when loading module: both, '{}' and '{}' exist",
                p0.display(),
                p1.display()
            );
        }
        (true, false) => p0,
        (false, true) => p1,
    };

    // Read the file's content.
    let content = {
        let map_err = |e: io::Error| {
            name_span
                .error(format!("error reading module file '{}'", name.as_str()))
                .note(e.to_string())
        };

        let mut file = File::open(&p).map_err(&map_err)?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(map_err)?;
        content
    };

    // Parse item in file.
    let tokens: TokenStream = content.parse().map_err(|e| name_span.error(format!("{:?}", e)))?;
    let mut iter = Iter::new(tokens);
    let (modules, trans_units) = parse_items(&mut iter, p.parent().unwrap())?;

    Ok(ast::Mod {
        name,
        modules,
        trans_units,
    })
}

/// Parses one translation unit from the given iterator.
///
/// ```
/// translation_unit :=
///     "unit" <term> [<unit_parameters>] [<return_type>] "{" <unit_body> "}"
///```
fn parse_trans_unit(iter: &mut Iter) -> Result<ast::TransUnit> {
    // Each translation unit starts with the `unit` keyword followed by a name.
    // The keyword was already eaten by the calling function.
    let name = iter.eat_term()?;

    // Check if there is a paramter list and parse it if that's the case.
    let params = match *iter.peek_curr()? {
        TokenTree { kind: TokenNode::Group(Delimiter::Parenthesis, _), .. } => {
            let param_group = iter.eat_group_delimited_by(Delimiter::Parenthesis)?;
            let params = parse_unit_params(param_group.obj)?;
            Some(params)
        }
        _ => None,
    };

    // Check if there is a custom return type and parse it if that's the case.
    let return_type = match *iter.peek_curr()? {
        TokenTree { kind: TokenNode::Op('-', spacing), span } => {
            // Consume the '->' operator and emit errors if it isn't found
            // correctly.
            if eq_spacing(spacing, Spacing::Alone) {
                return err!(span, "expected '->' or '{{', found '-'");
            }
            iter.eat_op_if('-')?;
            iter.eat_op_if('>')?;

            // Parse actual return type.
            Some(parse_type(iter)?)
        }
        _ => None,
    };

    // Parse the body or emit errors if the next token is not a group delimited
    // by a brace.
    let body = match iter.eat_curr()? {
        TokenTree { kind: TokenNode::Group(Delimiter::Brace, ts), .. } => {
            parse_unit_body(ts)?
        }
        ref other if return_type.is_some() => {
            return err!(
                other.span,
                "expected block delimited by '{{', found '{}'",
                other
            );
        }
        ref other if params.is_some() => {
            return err!(
                other.span,
                "expected '->' or a block delimited by '{{', found '{}'",
                other
            );
        }
        ref other => {
            return err!(
                other.span,
                "expected '(', or '->' or a block delimited by '{{', found '{}'",
                other
            );
        }
    };

    Ok(ast::TransUnit {
        name,
        params,
        return_type,
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
/// all potentially valid tokens (all except `,` and '{') to a string buffer.
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
            Ok(&TokenTree { kind: TokenNode::Group(Delimiter::Brace, _), ..}) => break,
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
        {
            let (spacing, span) = iter.eat_op_if('=')?;
            if eq_spacing(spacing, Spacing::Alone) {
                return err!(span, "expected '=>', found '='");
            }
            iter.eat_op_if('>')?;
        }

        // ... followed by the actual body.
        let body = parse_arm_body(&mut iter)?;

        // Maybe eat comma, if haven't reached the end
        if !iter.is_exhausted() {
            if body.obj.is_raw_block() {
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
    if let Ok((_, span)) = iter.eat_op_if('_') {
        // The pattern is a wildcard pattern.
        Ok(ast::ArmPattern::Underscore(span))
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
            let mut inner_iter = Iter::new(region_group.obj);
            let region = inner_iter.eat_term()?;
            if let Ok(tok) = inner_iter.eat_curr() {
                return err!(tok.span, "didn't expect token '{:?}' in matcher", tok);
            }

            Ok(ast::ArmPattern::WithRegion {
                lang,
                region,
            })
        }
    }
}

/// Parses the body of one arm.
fn parse_arm_body(iter: &mut Iter) -> Result<Spanned<ast::ArmBody>> {
    // If we encounter a group next, we know the body is raw Rust.
    if iter.peek_curr()?.kind.is_group() {
        // Raw Rust body
        let group = iter.eat_group_delimited_by(Delimiter::Brace)?;
        Ok(Spanned::new(ast::ArmBody::Raw(group.obj), group.span))
    } else {
        // A standard body consisting of a single literal.
        let lit = iter.eat_literal()?;
        match lit.obj.parse_string() {
            Some(s) => Ok(Spanned::new(ast::ArmBody::Str(s), lit.span)),
            None => err!(lit.span, "expected string literal, found '{}'", lit.obj),
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
    fn eat_term(&mut self) -> Result<Ident> {
        match self.eat_curr()? {
            TokenTree { kind: TokenNode::Term(term), span } => {
                Ok(Ident::new(term, span))
            }
            other => {
                err!(other.span, "expected an identifier, found '{}'", other)
            },
        }
    }

    /// Consumes the next token. If that token is not a term with the value
    /// `expected`, an error is returned.
    fn eat_keyword(&mut self, expected: &str) -> Result<()> {
        let keyword = self.eat_term();

        match keyword {
            Ok(keyword) if keyword.as_str() == expected => Ok(()),
            Ok(keyword) => {
                err!(
                    keyword.span().unwrap(),
                    "expected keyword '{}', found identifier '{}'",
                    expected,
                    keyword
                )
            }
            Err(e) => {
                note!(e, "expected keyword '{}'", expected)
            }
        }
    }

    /// Consumes and returns the next tt if it is a `Group`. Otherwise an `Err`
    /// is returned.
    fn eat_group(&mut self) -> Result<(Delimiter, Spanned<TokenStream>)> {
        match self.eat_curr()? {
            TokenTree { kind: TokenNode::Group(delim, ts), span } => {
                Ok((delim, Spanned::new(ts, span)))
            }
            other => err!(other.span, "expected block (delimited token stream), found '{}'", other),
        }
    }

    /// Consumes and returns the next tt if it is a `Group` delimited by the
    /// given `delim`. Otherwise an `Err` is returned.
    fn eat_group_delimited_by(&mut self, delim: Delimiter) -> Result<Spanned<TokenStream>> {
        let (actual_delim, group) = self.eat_group()?;
        if eq_delim(delim, actual_delim) {
            Ok(group)
        } else {
            err!(
                group.span,
                "expected block delimited by '{:?}', found block delimited by '{:?}'",
                delim,
                actual_delim
            )
        }
    }

    /// Consumes and returns the next tt if it is a `Literal`. Otherwise an
    /// `Err` is returned.
    fn eat_literal(&mut self) -> Result<Spanned<Literal>> {
        match self.eat_curr()? {
            TokenTree { kind: TokenNode::Literal(lit), span } => Ok(Spanned::new(lit, span)),
            other => err!(other.span, "expected a literal, found '{}'", other),
        }
    }

    /// Consumes and returns the next tt if it equals the given operator.
    /// Otherwise an `Err` is returned.
    fn eat_op_if(&mut self, op: char) -> Result<(Spacing, Span)> {
        let out = match *self.peek_curr()? {
            TokenTree { kind: TokenNode::Op(found_op, spacing), span } if found_op == op => {
                (spacing, span)
            }
            ref other => return err!(other.span, "expected '{}', found '{}'", op, other),
        };
        self.bump();
        Ok(out)
    }

    /// Peeks onto the current tt. Returns an error if there is no next tt.
    fn peek_curr(&mut self) -> Result<&TokenTree> {
        self.0.peek().ok_or(Diagnostic::new(Level::Error, "unexpected EOF"))
    }

    /// Consumes and returns the current tt or an error if there is no next tt.
    fn eat_curr(&mut self) -> Result<TokenTree> {
        self.0.next().ok_or(Diagnostic::new(Level::Error, "unexpected EOF"))
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
