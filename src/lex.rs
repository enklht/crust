use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("unknown token: {0}")]
    UnknownToken(String),
}
