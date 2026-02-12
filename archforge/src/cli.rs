use crate::ai::AiProvider;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// AI-powered TUI for PKGBUILD generation and AUR management
#[derive(Parser, Debug)]
#[command(name = "archforge")]
#[command(author, version)]
#[command(after_help = r#"Tips:
  • Set CHUTES_API_KEY env var for AI generation
  • Run 'archforge status' to check environment
  • Use 'archforge generate "package description"' for quick start
  • Use 'archforge interactive' for full TUI experience"#)]
#[command(long_about = r#"ArchForge - AI-powered TUI for PKGBUILD generation and AUR management

A modern tool for Arch Linux package maintainers with:
  ✨ AI-powered PKGBUILD generation from natural language
  🔍 AUR search and package information
  ✅ PKGBUILD validation with namcap
  🚀 Build and deployment automation

Quick Start:
  archforge generate 'rust http server'
  archforge search 'vim plugins'
  archforge validate ./mypkg
  archforge interactive

Environment:
  CHUTES_API_KEY    AI provider API key (default: chutes)
  ARCHFORGE_CONFIG  Config file path (default: ~/.config/archforge/config.toml)"#)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Config file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a PKGBUILD from natural language
    #[command(alias = "gen")]
    Generate {
        /// Description of the package to generate
        description: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Don't show preview, just output
        #[arg(short, long)]
        quiet: bool,

        /// AI provider to use (default: chutes)
        #[arg(short, long, value_enum, default_value = "chutes")]
        ai_provider: AiProvider,

        /// API key for the AI provider (can also use CHUTES_API_KEY env var)
        #[arg(long)]
        api_key: Option<String>,
    },

    /// Build a package
    Build {
        /// Package name or path to PKGBUILD
        package: String,

        /// Install after building
        #[arg(short, long)]
        install: bool,

        /// Skip dependency checks
        #[arg(long)]
        nodeps: bool,
    },

    /// Search AUR for packages
    #[command(alias = "s")]
    Search {
        /// Search query
        query: String,

        /// Show results in JSON format
        #[arg(long)]
        json: bool,

        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },

    /// Show package information
    #[command(alias = "i")]
    Info {
        /// Package name
        package: String,
    },

    /// Deploy package to AUR, Docker, Flatpak, or Nix
    Deploy {
        /// Package name or path
        package: String,

        /// Deployment target
        #[arg(short, long, value_enum)]
        target: Option<DeployTarget>,

        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },

    /// Launch interactive TUI
    Interactive {
        /// Don't auto-load model on startup
        #[arg(long)]
        no_model: bool,
    },

    /// Initialize a new project
    Init {
        /// Project name
        name: String,

        /// Template to use
        #[arg(short, long, default_value = "basic")]
        template: String,

        /// Project directory
        #[arg(short, long)]
        directory: Option<PathBuf>,
    },

    /// Validate PKGBUILD using namcap
    #[command(alias = "v")]
    Validate {
        /// Path to package directory or PKGBUILD
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Validate .SRCINFO instead of PKGBUILD
        #[arg(long)]
        srcinfo: bool,

        /// Check dependencies only
        #[arg(long)]
        dependencies: bool,
    },

    /// Manage swarm network
    #[command(subcommand)]
    Swarm(SwarmCommands),

    /// Show status
    Status,

    /// Cache management
    #[command(subcommand)]
    Cache(CacheCommands),
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum DeployTarget {
    /// Upload to AUR
    Aur,
    /// Build Docker image
    Docker,
    /// Build Flatpak bundle
    Flatpak,
    /// Generate Nix flake
    Nix,
}

#[derive(Subcommand, Debug)]
pub enum SwarmCommands {
    /// Start swarm networking
    Start {
        /// Enable telemetry sharing
        #[arg(long)]
        telemetry: bool,
    },
    /// Stop swarm networking
    Stop,
    /// Show connected peers
    Peers,
    /// Sync with a peer
    Sync {
        /// Peer address
        address: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// Show cache statistics
    Stats,
    /// Clear model cache
    Models,
    /// Clear build cache
    Builds,
    /// Clear all caches
    All,
}