//! The miniature tools: every one opens, in both systems.

use super::harness::Ui;
use crate::views::miniature::MiniatureTab;
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::PaintSystem;

async fn open_every_tool(ui: &mut Ui, system: PaintSystem) -> usize {
    ui.click_in("sidebar", "Figurine & Weathering").await;
    let mut opened = 0;
    for (_, tabs) in MiniatureTab::grouped(system, Locale::Fr, "") {
        for tab in tabs {
            let label = tab.label(Locale::Fr);
            ui.click_in("mini-nav", label).await;
            assert_eq!(
                MiniatureTab::from_idx(ui.peek(|s| *s.miniature_tab.peek())),
                tab,
                "{label} is the open tool"
            );
            let header = ui.find_class("mini-content-header").expect("tool header");
            assert!(
                header.text().contains(label),
                "header of {label}: {}",
                header.text()
            );
            opened += 1;
        }
    }
    opened
}

#[tokio::test(flavor = "current_thread")]
async fn every_tool_opens_in_oil() {
    let mut ui = Ui::studio().await;
    assert_eq!(
        open_every_tool(&mut ui, PaintSystem::Oil).await,
        MiniatureTab::ALL.len()
    );
}

#[tokio::test(flavor = "current_thread")]
async fn every_tool_opens_in_acrylic() {
    let mut ui = Ui::studio().await;
    ui.click_in("system-segment", "Acrylique").await;
    assert_eq!(
        open_every_tool(&mut ui, PaintSystem::Acrylic).await,
        MiniatureTab::ALL.len()
    );
    ui.assert_shows("Finition à l'huile").await;
}

#[tokio::test(flavor = "current_thread")]
async fn tool_filter_keeps_matching_tools() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Figurine & Weathering").await;
    ui.type_into("Filtrer les outils…", "vernis").await;
    let nav = ui.find_class("mini-nav").expect("nav");
    let items: Vec<String> = nav
        .descendants()
        .into_iter()
        .filter(|e| e.has_class("mini-nav-item"))
        .map(|e| e.text())
        .collect();
    assert_eq!(items, vec!["🪞 Vernis".to_string()]);
}
