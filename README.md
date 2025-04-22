# AI-Driven Command-Line Assistant

A Rust-based AI assistant that leverages the Ollama API to perform file operations and Markdown conversions through natural language chat interactions.

## What It Does
This project creates an interactive command-line interface where users can:  
- Read/edit/list files and directories  
- Convert Markdown content to HTML  
- Interact with AI models (default: `cogito:14b`) for smarter operations  
- Use pre-defined tools via natural language commands  

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org) (with Cargo)
- [Ollama](https://ollama.com) installed and running locally

### Installation
```bash
git clone https://github.com/joshholt/local-agent.git local-agent
cd local-agent
cargo build --release
```

### Running the Assistant
```bash
cargo run --release [OPTIONS]
```

## Usage
1. Start the assistant:
```bash
cargo run --release [OPTIONS]
```
2. Enter commands at the `You: >` prompt. Type `exit` to quit.

### Available Commands/Tools
The AI understands these operations:
- **File Reading**:  
  `read [file-path]` - Returns file contents  
- **File Editing**:  
  `edit [file-path] replace "[old-text]" with "[new-text]"`  
- **Directory Listing**:  
  `list [directory-path]` (default: current directory)  
- **Markdown Conversion**:  
  `convert "[markdown text]" to html`  

### Command-Line Options
- `-m, --model-name [MODEL]` - Specify Ollama model (default: `cogito:14b`)
- `-d, --debug` - Enable debug logging
- `-c, --ctx-size [SIZE]` - Set context window size (default: 20000)

## Contributing
1. Fork the repository
2. Create your feature branch (`git checkout -b feature/X`)
3. Commit changes (`git commit -am 'Add feature X'`)
4. Push to the branch (`git push origin feature/X`)
5. Create a new Pull Request

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details.
