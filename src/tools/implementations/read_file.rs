use ollama_rs::generation::tools::Tool;
use schemars::JsonSchema;
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncReadExt;

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    #[schemars(description = "The relative path to a file to be read.")]
    path: String,
}

pub struct FileReader {}

impl Tool for FileReader {
    type Params = Params;

    fn name() -> &'static str {
        "read_file"
    }

    fn description() -> &'static str {
        "Read and return the contents of a file at the given path only when the contents of a file is needed. The given path must not be a directory."
    }

    async fn call(
        &mut self,
        parameters: Self::Params,
    ) -> Result<String, Box<dyn Error + Sync + Send>> {
        // Convert incomming path String to a PathBuf
        let path = PathBuf::from(&parameters.path);
        let metadata = fs::metadata(&path).await?;

        if !metadata.is_file() {
            return Err(format!("Path {} is not a file.", path.display()).into());
        }

        // Read the file to a string.
        let mut reader = fs::OpenOptions::new().read(true).open(&path).await?;
        let mut contents = String::new();
        reader.read_to_string(&mut contents).await?;

        // If the file contents are empty, return an error stating that the file is empty.
        if contents.is_empty() {
            return Err(format!("File at path {} is empty.", path.display()).into());
        }

        Ok(contents)
    }
}
