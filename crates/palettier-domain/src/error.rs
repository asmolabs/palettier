use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainError {
    #[error("Code hexadécimal invalide : {0}")]
    InvalidHex(String),
    #[error("Argument invalide : {0}")]
    InvalidArgument(String),
    #[error("Erreur de format CSV dans {location} : {message}")]
    CsvError { location: String, message: String },
    #[error("Erreur de persistance : {0}")]
    Storage(String),
    #[error("Erreur de persistance : {0}")]
    Persistence(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
