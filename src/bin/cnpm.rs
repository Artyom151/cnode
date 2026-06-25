use cnode_lib::cli;
use clap::Parser;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = cli::Cli::parse();
    if let Err(e) = cli::execute(args).await {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
