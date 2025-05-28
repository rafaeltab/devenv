use std::collections::HashMap;

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct BashScriptPlaceholderName(pub String);

/// A segment of a BashScriptTemplate, either literal text or a placeholder.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptSpan {
    LiteralText(String),
    InputPlaceholder { name: BashScriptPlaceholderName },
}

/// Represents the template for a bash script with placeholders for inputs.
#[derive(Debug, Clone)]
pub struct BashScriptTemplate {
    pub spans: Vec<ScriptSpan>,
}

impl BashScriptTemplate {
    /// Resolves the template into an executable script string using provided answers.
    pub fn resolve(
        &self,
        values: HashMap<BashScriptPlaceholderName, String>,
    ) -> Result<String, Vec<BashScriptResolveErrors>> {
        let (oks, errs): (Vec<String>, Vec<BashScriptResolveErrors>) = self
            .spans
            .iter()
            .map(|step| match step {
                ScriptSpan::LiteralText(text) => Ok(text.clone()),
                ScriptSpan::InputPlaceholder { name } => match values.get(name) {
                    Some(value) => Ok(value.clone()),
                    None => Err(BashScriptResolveErrors::PlaceholderValueNotFound(
                        name.clone(),
                    )),
                },
            })
            .partition_result();

        if errs.is_empty() {
            return Ok(oks.join(""));
        }
        Err(errs)
    }
}

pub enum BashScriptResolveErrors {
    PlaceholderValueNotFound(BashScriptPlaceholderName),
}
