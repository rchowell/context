use clap::Parser;
use context::cli::{execute, map_exit_code, Cli};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match execute(cli).await {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(map_exit_code(false, Some(&e)));
        }
    }
}
