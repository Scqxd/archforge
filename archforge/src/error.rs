use thiserror::Error;

pub type Result<T> = std::result::Result<T, ArchforgeError>;

#[derive(Debug, Error)]
pub enum ArchforgeError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Model(#[from] ModelError),

    #[error(transparent)]
    Build(#[from] BuildError),

    #[error(transparent)]
    Network(#[from] NetworkError),

    #[error(transparent)]
    Deploy(#[from] DeployError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    NotFound(String),

    #[error("Invalid config format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Config I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Model not found: {0}")]
    NotFound(String),

    #[error("Failed to load model: {0}")]
    LoadError(String),

    #[error("Model inference error: {0}")]
    InferenceError(String),

    #[error("Tokenizer error: {0}")]
    TokenizerError(String),

    #[error("Out of memory")]
    OutOfMemory,
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Makepkg failed: {0}")]
    MakepkgFailed(String),

    #[error("Missing dependency: {0}")]
    MissingDependency(String),

    #[error("Build cancelled")]
    Cancelled,

    #[error("Build timeout")]
    Timeout,

    #[error("Invalid PKGBUILD: {0}")]
    InvalidPkgbuild(String),

    #[error("Package not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Sync failed: {0}")]
    SyncFailed(String),

    #[error("Discovery failed: {0}")]
    DiscoveryFailed(String),
}

#[derive(Debug, Error)]
pub enum DeployError {
    #[error("AUR upload failed: {0}")]
    AURFailed(String),

    #[error("Authentication required")]
    AuthRequired,

    #[error("Container build failed: {0}")]
    ContainerFailed(String),

    #[error("VCS error: {0}")]
    VCSError(String),
}