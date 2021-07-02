use serde::{Deserialize, Serialize};

pub mod css_selector;
pub mod html_prettify;
pub mod regex_replacer;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Editor {
    CssSelector(css_selector::CssSelector),
    HtmlPrettify,
    RegexReplacer(regex_replacer::RegexReplacer),
}

impl Editor {
    pub fn is_valid(&self) -> anyhow::Result<()> {
        match &self {
            Editor::CssSelector(e) => e.is_valid()?,
            Editor::HtmlPrettify => {}
            Editor::RegexReplacer(e) => e.is_valid()?,
        }
        Ok(())
    }

    pub fn apply(&self, input: &str) -> anyhow::Result<String> {
        match &self {
            Editor::CssSelector(e) => e.apply(input),
            Editor::HtmlPrettify => html_prettify::prettify(input),
            Editor::RegexReplacer(e) => Ok(e.replace_all(input)?.to_string()),
        }
    }
}
