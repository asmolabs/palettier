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

#[tokio::test(flavor = "current_thread")]
async fn switching_to_acrylic_keeps_a_valid_palette_when_oil_had_more_palettes() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Figurine & Weathering").await;
    // Choose the 3rd oil palette (index 2: Palette Zorn -> 4 tubes), then switch
    // to Acrylic (2 palettes): must clamp to a palette instead of falling back to all 1,427 pots.
    ui.type_into("Palette active pour le calcul des mélanges", "2")
        .await;
    ui.assert_shows("4 tubes disponibles").await;
    ui.click_in("system-segment", "Acrylique").await;
    ui.assert_hides("1427 tubes disponibles").await;
    ui.assert_shows("13 tubes disponibles").await;
}
