use proc_macro::Span;

use Result;
use ast::{self, Ident};


#[derive(Debug, Clone, Copy)]
pub struct Spanned<T> {
    pub obj: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(obj: T, span: Span) -> Self {
        Self { obj, span }
    }
}


macro_rules! err {
    ($span:expr, $fmt:expr $(, $arg:expr)* ) => {
        Err(
            $span.error(
                format!($fmt $(, $arg)*)
            )
        )
    }
}

macro_rules! note {
    ($span:expr, $fmt:expr $(, $arg:expr)* ) => {
        Err(
            $span.note(
                format!($fmt $(, $arg)*)
            )
        )
    }
}



/// Holds information about which locale-pattern were already exhausted.
///
/// Is used to check for unreachable patterns, and to check whether a match
/// has been exhausted.
#[derive(Debug)]
pub struct PatternUsage {
    root: UsageNode<Ident>,
}

impl PatternUsage {
    /// All idents in the given `LocaleDef` need to have valid spans!
    pub fn new(locale: &ast::LocaleDef) -> Self {
        let children = locale.langs.iter().map(|lang| {
            let children = lang.regions.iter().map(|reg_name| {
                UsageNode {
                    used: false,
                    children: vec![],
                    data: *reg_name,
                }
            }).collect();

            UsageNode {
                used: false,
                children,
                data: lang.name,
            }
        }).collect();

        Self {
            root: UsageNode {
                used: false,
                children,
                data: Ident::internal("Locale"),
            }
        }
    }

    /// Returns true if the whole pattern was exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.root.is_used()
    }

    /// Checks if the given language can still be used. If that language has
    /// been exhausted already, an error is returned. Otherwise the language
    /// is marked as used.
    pub fn use_lang(&mut self, lang: &str) -> Result<()> {
        let is_exhausted = self.is_exhausted();
        let lang_node = self.lang_mut(lang);

        if lang_node.is_used() || is_exhausted {
            err!(
                lang_node.data.span().unwrap(),
                "unreachable pattern '{}'",
                lang
            )
        } else {
            lang_node.used = true;
            Ok(())
        }
    }

    /// Checks if the given language-region pair can still be used. If that
    /// language-region pair has been used already, an Error is returned.
    /// Otherwise the pair is marked as used.
    pub fn use_region(&mut self, lang: &str, region: &str) -> Result<()> {
        let is_exhausted = self.is_exhausted();

        let lang_node = self.lang_mut(lang);
        let is_lang_used = lang_node.is_used();

        let region_node = lang_node.children.iter_mut()
            .find(|r| r.data.as_str() == region)
            .unwrap();

        if region_node.is_used() || is_lang_used || is_exhausted {
            err!(
                region_node.data.span().unwrap(),
                "unreachable pattern '{}({})'",
                lang,
                region
            )
        } else {
            region_node.used = true;
            Ok(())
        }
    }

    /// If the pattern has been exhausted already, an error is returned.
    /// Otherwise the whole pattern is set as used.
    ///
    /// The `binding` parameter is only useful for the error message. Pass
    /// `None` if the binding was a underscore, and `Some(name)` if the pattern
    /// was a binding to `name`.
    pub fn use_wildcard(&mut self, span: Span, binding: Option<&str>) -> Result<()> {
        if self.is_exhausted() {
            err!(
                span,
                "unreachable pattern '{}': match is already is_exhausted",
                binding.unwrap_or("_")
            )
        } else {
            self.root.used = true;
            Ok(())
        }
    }

    fn lang_mut(&mut self, lang: &str) -> &mut UsageNode<Ident> {
        self.root.children.iter_mut()
            .find(|l| l.data.as_str() == lang)
            .unwrap()
    }
}

#[derive(Debug)]
struct UsageNode<T> {
    used: bool,
    children: Vec<UsageNode<T>>,
    data: T,
}

impl<T> UsageNode<T> {
    fn is_used(&self) -> bool {
        self.used || (
            !self.children.is_empty()
            && self.children.iter().all(|c| c.is_used())
        )
    }

}
