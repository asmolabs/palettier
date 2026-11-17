//! Error types and result definitions for the domain layer.

use crate::i18n::Locale;
use thiserror::Error;

/// Domain error variants covering validation, calculation, parsing, and persistence issues.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainError {
    /// The supplied hex string does not represent a valid RGB hex color.
    #[error("Code hexadécimal invalide : {0}")]
    InvalidHex(String),

    /// An input parameter failed business validation rules.
    #[error("Argument invalide : {0}")]
    InvalidArgument(String),

    /// A parsing failure occurred while loading catalog or reference CSV data.
    #[error("Erreur de format CSV dans {location} : {message}")]
    CsvError {
        /// File or record identifier where the error occurred.
        location: String,
        /// Detail of the parsing problem.
        message: String,
    },

    /// An external service or network operation failed (e.g. AI provider HTTP call).
    #[error("Erreur de service externe : {0}")]
    Service(String),

    /// A storage or database persistence invariant was violated.
    #[error("Erreur de persistance : {0}")]
    Persistence(String),
}

impl DomainError {
    /// Formats this error message in the requested [`Locale`].
    ///
    /// # Example
    /// ```
    /// use palettier_domain::error::DomainError;
    /// use palettier_domain::i18n::Locale;
    ///
    /// let err = DomainError::InvalidHex("ZZZ".to_string());
    /// assert_eq!(err.message(Locale::En), "Invalid hexadecimal code: ZZZ");
    /// ```
    pub fn message(&self, locale: Locale) -> String {
        match (locale, self) {
            (Locale::Fr, _) => self.to_string(),
            (Locale::En, Self::InvalidHex(hex)) => format!("Invalid hexadecimal code: {hex}"),
            (Locale::En, Self::InvalidArgument(msg)) => format!("Invalid argument: {msg}"),
            (Locale::En, Self::CsvError { location, message }) => {
                format!("CSV format error in {location}: {message}")
            }
            (Locale::En, Self::Service(msg)) => format!("External service error: {msg}"),
            (Locale::En, Self::Persistence(msg)) => format!("Persistence error: {msg}"),
        }
    }
}

/// Specialized [`Result`] type for Palettier domain operations.
pub type DomainResult<T> = Result<T, DomainError>;
