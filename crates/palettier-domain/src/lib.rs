//! Core domain logic for Palettier: Kubelka-Munk colorimetry, paint mix search,
//! drying time modeling, fat-over-lean validation, and workbench readiness tracking.
//!
//! # Architecture
//!
//! The domain crate is strictly independent of UI frameworks (Dioxus) and storage
//! implementations (SeaORM, SQLite). It defines:
//! - **Pure entities and value types**: [`color::Rgb`], [`paint::Paint`], [`paint::PaintSystem`], [`project::Project`].
//! - **Physical & optical models**: Subtractive color mixing via Kubelka-Munk coefficients,
//!   perceptual distance via [`color::delta_e_2000`], and drying models ([`drying::DryingTimeService`]).
//! - **Ports**: Trait contracts for persistence and IO in [`port`].
//!
//! # Example: Color Distance and Hex Parsing
//!
//! ```
//! use palettier_domain::color::{Rgb, delta_e_2000};
//!
//! let red = Rgb::of_hex("#FF0000").unwrap();
//! let darker_red = Rgb::of_hex("#CC0000").unwrap();
//! let distance = delta_e_2000(red, darker_red);
//! assert!(distance > 0.0);
//! ```

/// Paint and pigment catalog data loading and classification.
pub mod catalog;

/// Color spaces (RGB, Lab), linear light conversion, CIEDE2000 metric, and optical averaging.
pub mod color;

/// Physical drying simulation, ambient climate impact, and fat-over-lean safety enforcement.
pub mod drying;

/// Domain error and result definitions.
pub mod error;

/// Printable workshop and studio reference guide generation.
pub mod guide;

/// HTML escaping and CSS color sanitization utilities.
pub mod html;

/// Localization primitives and bilingual (FR/EN) translation helpers.
pub mod i18n;

/// Image color extraction and spatial sampling abstractions.
pub mod image;

/// Miniature painting dedicated tools: batch painting, pin washes, glazing, and brush care.
pub mod miniature;

/// Subtractive color solver and recipe search across paint collections.
pub mod mix;

/// Core paint tube, pigment, medium, and paint system specifications.
pub mod paint;

/// Palettes and active paint mixes layout on the painter's palette.
pub mod palette;

/// Painting plans, zone breakdowns, and recipe enrichment.
pub mod plan;

/// Inversion-of-control persistence traits (Repository ports).
pub mod port;

/// Studio projects, miniature pieces, painted zones, and layer tracking.
pub mod project;

/// Step-by-step painting recipes and technique instructions.
pub mod recipe;

/// Workshop readiness tracking, open times, and real-time drying progress.
pub mod workbench;
