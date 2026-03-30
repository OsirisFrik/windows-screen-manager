use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod commands;
mod display;
mod profiles;
mod types;

#[derive(Parser)]
#[command(name = "wsm")]
#[command(about = "Windows Screen Manager — save and restore monitor configurations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Save current display configuration to a profile
    Save {
        /// Profile alias (stored in ~/.wsm-profiles)
        alias: String,
        /// Save to an external file instead of ~/.wsm-profiles
        #[arg(long)]
        output: Option<PathBuf>,
        /// Save as JSON instead of YAML
        #[arg(long)]
        json: bool,
    },
    /// Load a display configuration from a profile
    Load {
        /// Profile alias to load
        alias: String,
        /// Load from an external file instead of ~/.wsm-profiles
        #[arg(long)]
        source: Option<PathBuf>,
    },
    /// List all saved profiles
    List,
    /// Delete a profile
    Delete {
        /// Profile alias to delete
        alias: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Save { alias, output, json } => commands::save_profile(&alias, output.as_deref(), json),
        Commands::Load { alias, source } => commands::load_profile(&alias, source.as_deref()),
        Commands::List => commands::list_profiles(),
        Commands::Delete { alias } => commands::delete_profile(&alias),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
