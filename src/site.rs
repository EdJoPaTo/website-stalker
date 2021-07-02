use serde::{Deserialize, Serialize};
use url::Url;

use crate::editor::css_selector::CssSelector;
use crate::editor::regex_replacer::RegexReplacer;
use crate::editor::Editor;

#[derive(Debug, Deserialize, Serialize)]
pub struct Site {
    pub url: Url,
    pub extension: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub editors: Vec<Editor>,
}

impl Site {
    pub fn get_filename(&self) -> String {
        crate::url_filename::format(&self.url, &self.extension)
    }

    pub async fn stalk(&self, content: &str) -> anyhow::Result<String> {
        let mut content = content.to_string();

        for e in &self.editors {
            content = e.apply(&content)?;
        }

        Ok(content)
    }

    pub fn is_valid(&self) -> anyhow::Result<()> {
        if self.extension.is_empty() || self.extension.len() > 12 || !self.extension.is_ascii() {
            return Err(anyhow::anyhow!(
                "extension has to be a short ascii filename extension"
            ));
        }

        for e in &self.editors {
            e.is_valid()?;
        }
        Ok(())
    }

    pub fn examples() -> Vec<Site> {
        vec![
            Site {
                url: Url::parse("https://edjopato.de/post/").unwrap(),
                extension: "html".to_string(),
                editors: vec![
                    Editor::CssSelector(CssSelector {
                        selector: "article".to_string(),
                        remove: false,
                    }),
                    Editor::CssSelector(CssSelector {
                        selector: "a".to_string(),
                        remove: true,
                    }),
                    Editor::HtmlPrettify,
                    Editor::RegexReplacer(RegexReplacer {
                        pattern: "(Lesezeit): \\d+ \\w+".to_string(),
                        replace: "$1".to_string(),
                    }),
                ],
            },
            Site {
                url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
                extension: "txt".to_string(),
                editors: vec![],
            },
        ]
    }

    pub fn validate_no_duplicate(sites: &[Site]) -> Result<(), String> {
        // TODO: return url or something of specific duplicates
        let mut filenames = sites.iter().map(Site::get_filename).collect::<Vec<_>>();
        filenames.sort_unstable();
        let filename_amount = filenames.len();
        filenames.dedup();
        if filenames.len() == filename_amount {
            Ok(())
        } else {
            Err("Some sites are duplicates of each other".to_string())
        }
    }
}

#[test]
fn validate_finds_duplicates() {
    let sites = vec![
        Site {
            url: Url::parse("https://edjopato.de/post/").unwrap(),
            extension: "html".to_string(),
            editors: vec![],
        },
        Site {
            url: Url::parse("https://edjopato.de/robots.txt").unwrap(),
            extension: "txt".to_string(),
            editors: vec![],
        },
        Site {
            url: Url::parse("https://edjopato.de/post").unwrap(),
            extension: "html".to_string(),
            editors: vec![],
        },
    ];

    let result = Site::validate_no_duplicate(&sites);
    println!("{:?}", result);
    assert!(result.is_err());
}
