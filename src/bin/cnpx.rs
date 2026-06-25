use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cnpx <command> [args...]");
        process::exit(1);
    }

    let command = args[1].clone();
    let extra_args: Vec<String> = args[2..].to_vec();

    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Err(e) = rt.block_on(cnode_lib::cli::commands::exec::handle(command, extra_args)) {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}
