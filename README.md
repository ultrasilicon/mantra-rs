# mantra-rs

A multi-agent Rust code refactoring CLI tool using OpenAI.

## Quick Start

1. **Install Rust** (if you don't have it):
   https://rustup.rs/

2. **Install required tools:**
   ```sh
   rustup component add rustfmt clippy
   ```

3. **Set your OpenAI API key:**
   ```sh
   export OPENAI_API_KEY=your-key-here
   ```

4. **Build the app:**
   ```sh
   cargo build --release
   ```

5. **Run the app:**
   ```sh
   cargo run -- --repo <path-to-repo> --file <path-to-file> --refactor-type <type> --refactor-prompt "<prompt>"
   ```
   Example:
   ```sh
   cargo run -- --repo ./rag_examples/example_app --file rag_examples/example_app/src/main.rs --refactor-type extract-method --refactor-prompt "extract helper and name it foo()"
   ```

## Arguments
- `--repo`           Path to the Rust repo
- `--file`           Path to the file to refactor
- `--refactor-type`  Refactor type (e.g. extract-method)
- `--refactor-prompt`  Description of the refactor

That's it! The tool will guide you through the rest.
