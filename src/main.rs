mod tools;
use clap::Parser;
use ollama_rs::{
    Ollama, coordinator::Coordinator, generation::chat::ChatMessage, models::ModelOptions,
};
use std::error::Error;
use tokio::io::{AsyncWriteExt, stdout};
use tools::implementations::{
    DDGSearcher, EditFile, FileReader, ListFiles, MarkdownToHTML, Scraper,
};

const EXIT_COMMAND: &str = "exit";

#[derive(Parser)]
#[command(version, author, about, long_about = None)]
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
        .add_tool(FileReader {})
        .add_tool(ListFiles {})
        .add_tool(EditFile {})
        .add_tool(MarkdownToHTML {})
        .add_tool(Scraper {})
        .add_tool(DDGSearcher::new())
        .debug(cli.debug);

    // Implement an infinite loop that allows the users to supply text to provide to the assistant for responses.
    loop {
        stdout.write_all(b"\nWelcome to your Local AI Agent!\nI can perform many tasks for you like:\n\tRead Files\n\tList Files in a directory\n\tEdit/Create Files\n\tConvert Markdown to HTML\n\tSearch The Web to help provide answers.\nWhen you want to end your conversation, just type exit\nAre you ready to begin?").await?;
        stdout.flush().await?;
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
