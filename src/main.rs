use ollama_rs::{
    Ollama, coordinator::Coordinator, generation::chat::ChatMessage, models::ModelOptions,
};
use std::error::Error;
use std::path::PathBuf;
use tokio::fs;
// use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, stdout};

const MODEL_NAME: &str = "mistral-nemo:latest";
const DEBUG: bool = false;
const MODEL_CTX_SIZE: u64 = 20000;
const EXIT_COMMAND: &str = "exit";

/// Read and return the contents of a file at the given path only when the contents of a file is needed. The given path must not be a directory.
///
/// * path - The relative path to a file to be read.
#[ollama_rs::function]
async fn read_file(path: PathBuf) -> Result<String, Box<dyn Error + Send + Sync>> {
    // Check to see if the provided path is a file.
    // If it is not, then return an error stating that the path is not a file.
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

/// List files and directories at a given path. If no path is provided, lists files and directories in the current directory.
///
/// * path - The relative path to list files from. Defaults to current directory if not provided.
#[ollama_rs::function]
async fn list_files(path: PathBuf) -> Result<String, Box<dyn Error + Send + Sync>> {
    // Ensure that the given path is a directory or bail.
    if !path.is_dir() {
        return Err(format!("Path {} is not a directory.", path.display()).into());
    }

    let mut entries = fs::read_dir(&path).await?;
    let mut result = String::new();

    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if entry_path.is_dir() {
                result.push_str(&format!("{}/\n", name));
            } else {
                result.push_str(&format!("{}\n", name));
            }
        }
    }

    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create an Ollama client with default values (e.g. connecting to local host, etc...)
    let ollama = Ollama::default();
    // Create an empty Vec to hold the history of all chat messages.
    let history = vec![];
    // Create a mutable stdout so that we can send the end user messages from the assistant.
    let mut stdout = stdout();

    // Setup the coordinator so that assistant has access to the chat history and useful tools to help the user acheive their goals.
    let mut agent = Coordinator::new(ollama, MODEL_NAME.to_string(), history)
        .options(ModelOptions::default().num_ctx(MODEL_CTX_SIZE))
        .add_tool(read_file)
        .add_tool(list_files)
        .debug(DEBUG);

    // Implement an infinite loop that allows the users to supply text to provide to the assistant for responses.
    loop {
        stdout.write_all(b"\n\x1b[34mYou: >\x1b[0m ").await?;
        stdout.flush().await?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case(EXIT_COMMAND) {
            stdout.write_all(b"\nGoodbye.\n").await?;
            stdout.flush().await?;
            break;
        }

        match agent.chat(vec![ChatMessage::user(input.to_string())]).await {
            Ok(result) => {
                stdout
                    .write_all(
                        format!("\n\x1b[33mAssistant: >\x1b[0m {}", result.message.content)
                            .as_bytes(),
                    )
                    .await?;
                stdout.flush().await?;
            }
            Err(e) => eprintln!("Error during chat: {}", e),
        }
    }

    Ok(())
}
