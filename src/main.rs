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
