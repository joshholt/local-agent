use clap::Parser;
use ollama_rs::{
    Ollama, coordinator::Coordinator, generation::chat::ChatMessage, models::ModelOptions,
};
use std::error::Error;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, stdout};

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

/// Create a file at a given path and writes the given contents.
///
/// * path - The path to the file to create.
/// * contents - The contents to write to the give file.
async fn create_file(path: &PathBuf, contents: &str) -> Result<String, Box<dyn Error>> {
    match File::create(path).await {
        Ok(mut file) => {
            if let Err(e) = file.write_all(contents.as_bytes()).await {
                return Err(format!(
                    "Failed to write to file at {}, Reason: {}",
                    path.display(),
                    e
                )
                .into());
            } else {
                if let Err(e) = file.flush().await {
                    return Err(format!(
                        "Failed to flush contents to file at {}, Reason: {}",
                        path.display(),
                        e
                    )
                    .into());
                } else {
                    return Ok(format!("Successfully create file at {}", path.display()).into());
                }
            }
        }
        Err(e) => {
            return Err(
                format!("Failed to create file at {}, Reason: {}", path.display(), e).into(),
            );
        }
    }
}

/// Make edits to a text file. Replaces 'old_str' with 'new_str' in the given file. 'old_str' and 'new_str' Must be different from each other. If the file specified doesn't exist, it will be created.
///
/// * path - The path to the file
/// * old_str - Text to search for - must match eactly and must have only one match exactly
/// * new_str - Text to replace old_str with
#[ollama_rs::function]
async fn edit_file(
    path: PathBuf,
    old_str: String,
    new_str: String,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    // TODO: Add logic to edit or create a file.
    // If the file specified doesn't exits then it needs to be created
    // If the path for the file is not in the CWD and old_str is nil/empty then the directory structure will need to be created first.
    // Then the file can be created at the path given with the contents of new_str
    // If the file path given doesn't exist but is in the CWD and old_str is empty then we can just write the new file with the contens of new_str in CWD.
    // If the file exists and old_str and new_str are not empty and the result of replacing old_str with new_str doesn't yield the original contents of the files,
    //    then we write the replaced content back to the existing file.
    // Unless there are errors we return the string OK, otherwise we return Err with the reason as a string.
    if path.display().to_string().is_empty() || old_str == new_str {
        return Err(format!("Could not edit a file, input parameters were invalid.").into());
    }

    match fs::metadata(&path).await {
        Ok(_) => {
            // 1. Read the contents of the file
            let mut reader = fs::OpenOptions::new().read(true).open(&path).await?;
            let mut contents = String::new();
            reader.read_to_string(&mut contents).await?;

            // 2. Replace all old_str with new_str
            let new_contents = contents.replace(&old_str, &new_str);
            // 3. Check to see if new_contents is equal to old contents and old_str is not empty
            if contents == new_contents && !old_str.is_empty() {
                // 3a.   Return Err old_str not found in file.
                return Err(format!("old_str was not found in the file.").into());
            }
            // 4. Write new_contents to the files, if no errors occur writing to the file, return Ok with the str "OK"
            match fs::write(&path, new_contents.as_bytes()).await {
                // 4a.   If an error occurs writing the file return Err with the reason.
                Ok(_) => {
                    return Ok("OK".to_string());
                }
                Err(e) => {
                    return Err(format!(
                        "Failed to edit file at {}, reason: {}",
                        path.display(),
                        e
                    )
                    .into());
                }
            }
        }
        Err(_) => {
            // Since the files at the given path doesn't not exist we need to create it.
            // 1. Check the parent of the file path given
            if let Some(parent) = path.parent() {
                // We have a parent so we aren't working on a file in CWD.
                match fs::create_dir_all(&parent).await {
                    Ok(_) => match create_file(&path, &new_str).await {
                        Ok(s) => {
                            return Ok(s);
                        }
                        Err(e) => {
                            return Err(format!("{}", e).into());
                        }
                    },
                    Err(e) => {
                        return Err(format!("Failed to create required directory structure for new file. Reason: {}", e).into());
                    }
                }
            } else {
                match create_file(&path, &new_str).await {
                    Ok(s) => {
                        return Ok(s);
                    }
                    Err(e) => {
                        return Err(format!("{}", e).into());
                    }
                }
            }
            // 1a. If it is CWD then we can create the new file with the contents of new_str
            //       If there is an error creating the file, return Err with the string "failed to create file, {}", err
            //       Else return Ok with the string "Successfully created file {}", path
            // 1b. if it is not CWD then we have to create the dir structure first, then write the file with the contents of new_str
            //       If there is an error creating the directory structure, return Err with string "failed to create directory {}", err
            //       Else move on to writing the file above.
        }
    }
}

/// Convert Markdown to HTML
///
/// * contents - The Markdown text to be converted to HTML
#[ollama_rs::function]
async fn markdown_to_html(contents: String) -> Result<String, Box<Error + Send + Sync>> {
    if contents.is_empty() {
        return Err(format!("Cannot convert an empty string to HTML").into());
    }

    match markdown::to_html_with_options(&contents, &markdown::Options::gfm()) {
        Ok(result) => {
            return Ok(result);
        }
        Err(e) => {
            return Err(format!("Markdown to HTML conversion failed, Reason: {}", e).into());
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = String::from("cogito:14b"))]
    model_name: String,
    #[arg(short, long)]
    debug: bool,
    #[arg(short, long, default_value_t = 20000)]
    ctx_size: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Parse command line arguments.
    let cli = Cli::parse();
    // Create an Ollama client with default values (e.g. connecting to local host, etc...)
    let ollama = Ollama::default();
    // Create an empty Vec to hold the history of all chat messages.
    let history = vec![];
    // Create a mutable stdout so that we can send the end user messages from the assistant.
    let mut stdout = stdout();

    // Setup the coordinator so that assistant has access to the chat history and useful tools to help the user acheive their goals.
    let mut agent = Coordinator::new(ollama, cli.model_name.to_string(), history)
        .options(ModelOptions::default().num_ctx(cli.ctx_size))
        .add_tool(read_file)
        .add_tool(list_files)
        .add_tool(edit_file)
        .add_tool(markdown_to_html)
        .debug(cli.debug);

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
