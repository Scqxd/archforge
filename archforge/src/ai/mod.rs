//! AI providers for PKGBUILD generation

pub mod chutes;

pub use chutes::ChutesClient;

/// AI provider type
#[derive(Debug, Clone, Default, clap::ValueEnum)]
pub enum AiProvider {
    /// Chutes.io API (MiniMaxAI/MiniMax-M2.1-TEE)
    #[default]
    Chutes,
    /// Local model (not implemented yet)
    Local,
    /// OpenAI API (not implemented yet)
    Openai,
}