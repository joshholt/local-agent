use std::{error::Error, path::PathBuf};

use schemars::JsonSchema;
use serde::Deserialize;
use tokio::fs;

use ollama_rs::generation::tools::Tool;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(
        description = "The relative path to list files from. Defaults to current directory if not provided."
    )]
    path: String,
}

pub struct ListFiles {}

impl Tool for ListFiles {
    type Params = Params;

    fn name() -> &'static str {
        "list_files"
    }

    fn description() -> &'static str {
        "List files and directories at a given path. If no path is provided, lists files and directories in the current directory."
    }

    async fn call(
        &mut self,
        parameters: Self::Params,
    ) -> Result<String, Box<dyn Error + Sync + Send>> {
        let path = PathBuf::from(&parameters.path);
        if !path.is_dir() {
            return Err(format!("The provide path {} is not a directory, only directories can have their contents listed.", path.display()).into());
        }

        let mut entries = fs::read_dir(&path).await?;
        let mut result = String::new();

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                if entry_path.is_dir() {
                    result.push_str(&format!("{}/\n", name));
                } else {
                    result.push_str(&format!("{}\n", name));
                }
            }
        }

        Ok(result)
    }
}
