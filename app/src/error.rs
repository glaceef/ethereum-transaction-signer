use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::ConfigError),

    #[error(transparent)]
    Dotenv(#[from] dotenv::Error),

    #[error(transparent)]
    Ecdsa(#[from] k256::ecdsa::Error),

    #[error(transparent)]
    FromHex(#[from] hex::FromHexError),

    #[error("Invalid private key length (expected: 32, input: {0}).")]
    InvalidPrivateKeyLength(usize),
}
