//! What the screens do to the studio's data, without the screens.
//!
//! Each action is a plain async function: it takes `&AppServices` and the values the
//! painter typed or picked, builds the domain objects (`Project`, `PaletteMix`,
//! `RecipeStep`…), saves them and returns a `DomainResult` with what the screen needs
//! to show next. Actions never read or write Dioxus signals, never open a file dialog
//! and never spawn: the view gathers its inputs, calls the action inside one
//! `spawn(async …)`, then puts the result into its signals.
//!
//! That keeps the rules testable on `Database::in_memory()` (see `AppServices::in_memory`),
//! like the domain.
pub mod mixes;
pub mod projects;
