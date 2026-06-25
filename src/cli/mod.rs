pub mod commands;

use crate::error::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cnpm")]
#[command(about = "Node.js Package Manager written in Rust")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true)]
    pub registry: Option<String>,

    #[arg(long, global = true)]
    pub cache_dir: Option<PathBuf>,

    #[arg(long, global = true)]
    pub save_dev: bool,

    #[arg(long, global = true)]
    pub save_optional: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Install {
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,

        #[arg(short, long)]
        save: bool,
    },

    Uninstall {
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,

        #[arg(short, long)]
        save: bool,
    },

    Update {
        #[arg(value_name = "PACKAGE")]
        packages: Option<Vec<String>>,

        #[arg(short, long)]
        all: bool,
    },

    Search {
        #[arg(value_name = "QUERY")]
        query: String,

        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    Info {
        #[arg(value_name = "PACKAGE")]
        package: String,
    },

    List {
        #[arg(short, long)]
        depth: Option<usize>,

        #[arg(short, long)]
        global: bool,
    },

    Init {
        #[arg(short, long)]
        yes: bool,
    },

    Run {
        #[arg(value_name = "SCRIPT")]
        script: String,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    Cache {
        #[command(subcommand)]
        subcommand: CacheCommand,
    },

    Config {
        #[command(subcommand)]
        subcommand: ConfigCommand,
    },

    Clean,

    Audit,

    Version,

    New {
        #[arg(value_name = "NAME")]
        name: String,
    },

    Exec {
        #[arg(value_name = "COMMAND")]
        command: String,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum CacheCommand {
    Clean,
    Verify,
    List,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    Get { key: String },
    Set { key: String, value: String },
    List,
    Delete { key: String },
}

pub async fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Install { packages, save } => {
            commands::install::handle(packages, save, cli.registry, cli.cache_dir).await
        }
        Commands::Uninstall { packages, save } => {
            commands::uninstall::handle(packages, save).await
        }
        Commands::Update { packages, all } => {
            commands::update::handle(packages, all, cli.registry).await
        }
        Commands::Search { query, limit } => {
            commands::search::handle(query, limit, cli.registry).await
        }
        Commands::Info { package } => {
            commands::info::handle(package, cli.registry).await
        }
        Commands::List { depth, global } => {
            commands::list::handle(depth, global).await
        }
        Commands::Init { yes } => commands::init::handle(yes).await,
        Commands::Run { script, args } => {
            commands::run::handle(script, args).await
        }
        Commands::Cache { subcommand } => {
            commands::cache::handle(subcommand, cli.cache_dir).await
        }
        Commands::Config { subcommand } => {
            commands::config::handle(subcommand).await
        }
        Commands::Clean => commands::clean::handle(cli.cache_dir).await,
        Commands::Audit => commands::audit::handle().await,
        Commands::Version => commands::version::handle(),
        Commands::New { name } => commands::new::handle(name).await,
        Commands::Exec { command, args } => {
            commands::exec::handle(command, args).await
        }
    }
}
