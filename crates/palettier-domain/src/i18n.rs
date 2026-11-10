//! Internationalization (`Locale`) primitives shared across domain, AI, and UI crates.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported application languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Locale {
    #[default]
    Fr,
    En,
}

impl Locale {
    pub const ALL: [Locale; 2] = [Locale::Fr, Locale::En];

    /// ISO 639-1 language code (`"fr"` or `"en"`).
    #[inline]
    pub fn code(&self) -> &'static str {
        match self {
            Locale::Fr => "fr",
            Locale::En => "en",
        }
    }

    /// Short uppercase badge label (`"FR"` or `"EN"`).
    #[inline]
    pub fn badge(&self) -> &'static str {
        match self {
            Locale::Fr => "FR",
            Locale::En => "EN",
        }
    }

    /// Native language name.
    #[inline]
    pub fn display_name(&self) -> &'static str {
        match self {
            Locale::Fr => "Français",
            Locale::En => "English",
        }
    }

    /// Toggles between French and English.
    #[inline]
    pub fn toggle(&self) -> Self {
        match self {
            Locale::Fr => Locale::En,
            Locale::En => Locale::Fr,
        }
    }

    /// Selects the localized string slice for the active locale.
    #[inline]
    pub fn tr<'a>(&self, fr: &'a str, en: &'a str) -> &'a str {
        match self {
            Locale::Fr => fr,
            Locale::En => en,
        }
    }

    /// True if the current locale is French.
    #[inline]
    pub fn is_fr(&self) -> bool {
        matches!(self, Locale::Fr)
    }

    /// True if the current locale is English.
    #[inline]
    pub fn is_en(&self) -> bool {
        matches!(self, Locale::En)
    }
}

impl FromStr for Locale {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "en" | "english" | "en-us" | "en-gb" => Ok(Locale::En),
            "fr" | "français" | "francais" | "fr-fr" => Ok(Locale::Fr),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}
