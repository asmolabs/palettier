//! Colour screens: the eyedropper, the painting plan assistant and the mixer.

mod assistant;
mod eyedropper;
mod eyedropper_groups;
mod mixer;
mod mixer_solver;

pub use assistant::AssistantView;
pub use eyedropper::EyedropperView;
pub use mixer::MixerView;

/// Shared with the HUD, which shows the same value studies.
pub(crate) use eyedropper::show_study;
