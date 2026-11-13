//! View tests: the real `StudioShell` runs in a headless `VirtualDom`, on an in-memory
//! database seeded like a first launch. A test clicks and types the way the painter
//! would (by visible text, placeholder or title) and reads back what the screen shows.
//!
//! No webview is involved, so CSS and JavaScript (`document::eval`, file dialogs) are
//! out of reach; the capture mode covers the look.

mod harness;
mod miniature;
mod reference;
mod shell;
mod workshop;
