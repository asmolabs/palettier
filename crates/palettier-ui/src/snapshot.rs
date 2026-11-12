//! Design snapshots: with `PALETTIER_SNAPSHOT_DIR` set, the app walks through every
//! screen once and writes the rendered HTML of each to that folder, then quits.
//!
//! The page is a still image (no interaction), but it carries the real markup and the
//! theme, so it can be opened in any browser to review the design without a screen
//! recording permission. Not used in normal runs.

use crate::state::{AppState, Section, ThemeMode};
use palettier_domain::i18n::Locale;
use dioxus::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

const SECTIONS: [Section; 11] = [
    Section::Today,
    Section::Projects,
    Section::Palettes,
    Section::Mixer,
    Section::Eyedropper,
    Section::Assistant,
    Section::Catalog,
    Section::Drying,
    Section::Recipes,
    Section::Miniature,
    Section::Settings,
];

pub fn target_dir() -> Option<PathBuf> {
    std::env::var_os("PALETTIER_SNAPSHOT_DIR").map(PathBuf::from)
}

pub fn run(mut state: AppState, dir: PathBuf) {
    spawn(async move {
        let _ = std::fs::create_dir_all(&dir);
        // Let the initial data load settle.
        tokio::time::sleep(Duration::from_millis(2500)).await;
        state.locale.set(Locale::Fr);
        for theme in [ThemeMode::Dark, ThemeMode::Light] {
            state.theme.set(theme);
            for (i, section) in SECTIONS.iter().enumerate() {
                state.section.set(*section);
                tokio::time::sleep(Duration::from_millis(900)).await;
                let name = format!("{}-{:02}-{:?}.html", theme.as_str(), i, section).to_lowercase();
                capture(&dir.join(name)).await;
            }
        }
        std::process::exit(0);
    });
}

async fn capture(path: &PathBuf) {
    let js = r#"
        const w = window.innerWidth, h = window.innerHeight;
        dioxus.send('<!-- viewport ' + w + 'x' + h + ' -->' + document.documentElement.outerHTML);
    "#;
    let mut eval = document::eval(js);
    if let Ok(html) = eval.recv::<String>().await {
        let _ = std::fs::write(path, format!("<!doctype html><meta charset=\"utf-8\">{html}"));
    }
}
