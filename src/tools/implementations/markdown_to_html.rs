use ollama_rs::generation::tools::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "The Markdown text to be converted to HTML.")]
    contents: String,
}

pub struct MarkdownToHTML {}

impl Tool for MarkdownToHTML {
    type Params = Params;

    fn name() -> &'static str {
        "markdown_to_html"
    }

    fn description() -> &'static str {
        "Convert Markdown to HTML"
    }

    async fn call(
        &mut self,
        parameters: Self::Params,
    ) -> Result<String, Box<dyn Error + Sync + Send>> {
        if parameters.contents.is_empty() {
            return Err(format!("Cannot convert an empty string to HTML").into());
        }

        match markdown::to_html_with_options(&parameters.contents, &markdown::Options::gfm()) {
            Ok(result) => {
                return Ok(result);
            }
            Err(e) => {
                return Err(format!("Markdown to HTML conversion failed, Reason: {}", e).into());
            }
        }
    }
}
