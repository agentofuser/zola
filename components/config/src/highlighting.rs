use lazy_static::lazy_static;
use syntect::dumps::from_binary;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

use crate::config::Config;
use syntect::html::{css_for_theme_with_class_style, ClassStyle};

lazy_static! {
    pub static ref SYNTAX_SET: SyntaxSet = {
        let ss: SyntaxSet =
            from_binary(include_bytes!("../../../sublime/syntaxes/newlines.packdump"));
        ss
    };
    pub static ref THEME_SET: ThemeSet =
        from_binary(include_bytes!("../../../sublime/themes/all.themedump"));
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HighlightSource {
    /// One of the built-in Zola syntaxes
    BuiltIn,
    /// Found in the extra syntaxes
    Extra,
    /// No language specified
    Plain,
    /// We didn't find the language in built-in and extra syntaxes
    NotFound,
}

pub struct SyntaxAndTheme<'config> {
    pub syntax: &'config SyntaxReference,
    pub syntax_set: &'config SyntaxSet,
    /// None if highlighting via CSS
    pub theme: Option<&'config Theme>,
    pub source: HighlightSource,
}

pub fn resolve_syntax_and_theme<'config>(
    language: Option<&'_ str>,
    config: &'config Config,
) -> SyntaxAndTheme<'config> {
    let theme = if config.markdown.highlight_theme != "css" {
        Some(&THEME_SET.themes[&config.markdown.highlight_theme])
    } else {
        None
    };

    if let Some(ref lang) = language {
        if let Some(ref extra_syntaxes) = config.markdown.extra_syntax_set {
            if let Some(syntax) = extra_syntaxes.find_syntax_by_token(lang) {
                return SyntaxAndTheme {
                    syntax,
                    syntax_set: extra_syntaxes,
                    theme,
                    source: HighlightSource::Extra,
                };
            }
        }
        // The JS syntax hangs a lot... the TS syntax is probably better anyway.
        // https://github.com/getzola/zola/issues/1241
        // https://github.com/getzola/zola/issues/1211
        // https://github.com/getzola/zola/issues/1174
        let hacked_lang = if *lang == "js" || *lang == "javascript" { "ts" } else { lang };
        if let Some(syntax) = SYNTAX_SET.find_syntax_by_token(hacked_lang) {
            SyntaxAndTheme {
                syntax,
                syntax_set: &SYNTAX_SET as &SyntaxSet,
                theme,
                source: HighlightSource::BuiltIn,
            }
        } else {
            SyntaxAndTheme {
                syntax: SYNTAX_SET.find_syntax_plain_text(),
                syntax_set: &SYNTAX_SET as &SyntaxSet,
                theme,
                source: HighlightSource::NotFound,
            }
        }
    } else {
        SyntaxAndTheme {
            syntax: SYNTAX_SET.find_syntax_plain_text(),
            syntax_set: &SYNTAX_SET as &SyntaxSet,
            theme,
            source: HighlightSource::Plain,
        }
    }
}

pub fn export_theme_css(theme_name: &str) -> String {
    let theme = &THEME_SET.themes[theme_name];
    css_for_theme_with_class_style(theme, ClassStyle::Spaced)
}
