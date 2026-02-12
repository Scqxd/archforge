//! AI providers for PKGBUILD generation

pub mod chutes;

pub use chutes::ChutesClient;

/// AI provider type
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum AiProvider {
    /// Chutes.io API (MiniMaxAI/MiniMax-M2.1-TEE)
    Chutes,
    /// Local model (not implemented yet)
    Local,
    /// OpenAI API (not implemented yet)
    Openai,
}

impl Default for AiProvider {
    fn default() -> Self {
        Self::Chutes
    }
}