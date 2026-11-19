#![forbid(unsafe_code)]

mod actions;
mod icons;
mod snapshot;
mod state;
mod theme;
#[cfg(test)]
mod ui_tests;
mod views;

use crate::state::AppServices;
use crate::theme::APP_CSS;
pub use crate::views::shell::{StudioShell, StudioShellProps};
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use palettier_data::default_db_dir;
use std::path::PathBuf;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "palettier=info,warn".into()),
        )
        .init();

    let window = WindowBuilder::new()
        .with_title("Palettier — Atelier de figurines")
        .with_inner_size(dioxus::desktop::LogicalSize::new(1460.0, 930.0))
        .with_min_inner_size(dioxus::desktop::LogicalSize::new(1160.0, 740.0));

    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window))
        .launch(App);
}

#[component]
fn App() -> Element {
    let mut services_sig = use_signal(|| None::<AppServices>);
    let mut init_error = use_signal(|| None::<String>);

    use_effect(move || {
        spawn(async move {
            let dir = default_db_dir();
            match AppServices::init(dir.clone()).await {
                Ok(svc) => services_sig.set(Some(svc)),
                Err(err) => {
                    let local = PathBuf::from(".").join(".palettier");
                    tracing::error!(
                        "Database initialization failed in {:?} ({}): {err}",
                        dir,
                        local.display()
                    );
                    init_error.set(Some(err.to_string()));
                }
            }
        });
    });

    rsx! {
        style { "{APP_CSS}" }
        if let Some(err) = init_error.read().clone() {
            div { class: "app-shell theme-dark u-ai-center u-jc-center u-p-40",
                div { class: "card u-maxw-560",
                    h2 { class: "card-title u-c-danger", "Studio initialization error / Erreur d'initialisation" }
                    p { "{err}" }
                }
            }
        } else if let Some(services) = services_sig.read().clone() {
            StudioShell { services }
        } else {
            div { class: "app-shell theme-dark u-ai-center u-jc-center",
                div { class: "muted", "Opening Palettier Studio… / Ouverture de l'atelier Palettier…" }
            }
        }
    }
}
