use clap::{Command, Parser, Subcommand};
use clap_complete::{Generator, Shell};

#[derive(Parser)]
#[command(name = "rift", version = "0.1.0", about = "Cross-engine game asset pipeline manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new rift.yml in the current directory
    Init {
        /// Engine to configure for (unity, godot, unreal, or all)
        #[arg(short, long, default_value = "all")]
        engine: String,
    },

    /// Run the pipeline once (scan, convert, validate, export)
    Run {
        /// Path to rift.yml (default: ./rift.yml)
        #[arg(short, long)]
        config: Option<String>,

        /// Force re-conversion of all assets (skip cache check)
        #[arg(short = 'C', long)]
        clean: bool,

        /// Print verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Watch source directory for changes and auto-run pipeline
    Watch {
        /// Path to rift.yml (default: ./rift.yml)
        #[arg(short, long)]
        config: Option<String>,

        /// API server port (default: 8910)
        #[arg(short, long, default_value = "8910")]
        port: u16,

        /// Enable the embedded API dashboard
        #[arg(short, long)]
        dashboard: bool,
    },

    /// Show pipeline status and asset database info
    Status {
        /// Path to rift database (default: .rift/state.db)
        #[arg(short, long)]
        db: Option<String>,

        /// Show detailed asset listing
        #[arg(short, long)]
        assets: bool,
    },

    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Generate completion script for the given shell
pub fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    clap_complete::generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
