//! The frame around every screen: navigation, shortcuts, system, language, theme.

use super::harness::Ui;
use crate::state::Section;
use dioxus::prelude::*;
use palettier_domain::port::SettingsRepository;

#[tokio::test(flavor = "current_thread")]
async fn sidebar_opens_every_screen() {
    let mut ui = Ui::studio().await;
    let screens = [
        ("Projets", "Projets & Pièces"),
        ("Palettes", "Palettes de travail"),
        ("Mélangeur", "Mélangeur & Simulateur Kubelka-Munk"),
        ("Pipette", "Pipette & Relevés sur référence"),
        ("Assistant IA", "Assistant de plan de peinture"),
        ("Figurine & Weathering", "Atelier Figurine & Modélisme"),
        ("Catalogue", "Catalogue des peintures à l'huile"),
        ("Séchage", "Calculateur de séchage à l'huile"),
        ("Recettes", "Recettes & séquences d'étapes"),
        ("Paramètres", "Paramètres & Sauvegardes"),
        ("Aujourd'hui", "Aujourd'hui à l'établi"),
    ];
    for (nav, heading) in screens {
        ui.click_in("sidebar", nav).await;
        ui.assert_shows(heading).await;
    }
}

#[tokio::test(flavor = "current_thread")]
async fn shortcuts_switch_screens_and_open_search() {
    let mut ui = Ui::studio().await;
    ui.press_cmd("2").await;
    assert_eq!(ui.peek(|s| *s.section.peek()), Section::Projects);
    ui.press_cmd("m").await;
    assert_eq!(ui.peek(|s| *s.section.peek()), Section::Miniature);
    ui.press_cmd("0").await;
    assert_eq!(ui.peek(|s| *s.section.peek()), Section::Settings);

    ui.press_cmd("k").await;
    ui.assert_shows("Palette de Commandes & Recherche Globale")
        .await;
    ui.press_cmd("k").await;
    ui.assert_hides("Palette de Commandes & Recherche Globale")
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn command_palette_finds_a_screen_and_goes_there() {
    let mut ui = Ui::studio().await;
    ui.click("Recherche globale & commandes (⌘K)").await;
    ui.type_into("Rechercher un outil", "séchage").await;
    ui.click_in("modal-backdrop", "Temps de Séchage").await;
    assert_eq!(ui.peek(|s| *s.section.peek()), Section::Drying);
    ui.assert_hides("Palette de Commandes & Recherche Globale")
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn painting_system_drives_the_screens_outside_a_project() {
    let mut ui = Ui::studio().await;
    ui.assert_shows("Tiroir huile : 680 / 680").await;

    ui.click_in("system-segment", "Acrylique").await;
    ui.assert_shows("Tiroir acrylique : 71 / 71").await;
    ui.click_in("sidebar", "Catalogue").await;
    ui.assert_shows("71 références").await;
    ui.assert_hides("Catalogue des peintures à l'huile").await;
    ui.click_in("sidebar", "Séchage").await;
    ui.assert_hides("Calculateur de séchage à l'huile").await;
    ui.click_in("sidebar", "Recettes").await;
    ui.assert_hides("à l'huile, vérifiez la règle du gras sur maigre")
        .await;

    // The choice is remembered in the settings.
    let saved = ui
        .services
        .settings
        .get(crate::state::AppState::KEY_PAINT_SYSTEM)
        .await
        .expect("settings readable");
    assert_eq!(saved.as_deref(), Some("ACRYLIC"));

    ui.click_in("system-segment", "Huile").await;
    ui.click_in("sidebar", "Catalogue").await;
    ui.assert_shows("Catalogue des peintures à l'huile").await;
}

#[tokio::test(flavor = "current_thread")]
async fn language_switch_translates_the_frame_and_back() {
    let mut ui = Ui::studio().await;
    ui.click_in("quick-toggles", "EN").await;
    ui.assert_shows("Oil drawer: 680 / 680").await;
    ui.assert_shows("Projects").await;
    ui.assert_hides("Tiroir huile").await;

    ui.click_in("sidebar", "Settings").await;
    ui.click("Français (FR)").await;
    ui.assert_shows("Tiroir huile : 680 / 680").await;
}

#[tokio::test(flavor = "current_thread")]
async fn theme_switch_changes_the_shell_class() {
    let mut ui = Ui::studio().await;
    let shell = ui.find_class("app-shell").expect("shell");
    assert!(shell.has_class("theme-dark"));

    ui.click_in("sidebar", "Paramètres").await;
    ui.click("☀️ Mode Jour").await;
    let shell = ui.find_class("app-shell").expect("shell");
    assert!(
        shell.has_class("theme-light"),
        "classes: {:?}",
        shell.attr("class")
    );
    ui.assert_shows("Jour · Français").await;
}

#[tokio::test(flavor = "current_thread")]
async fn command_palette_follows_the_system_and_the_language() {
    let mut ui = Ui::studio().await;
    ui.click_in("system-segment", "Acrylique").await;
    ui.press_cmd("k").await;
    ui.type_into("Rechercher un outil", "catalogue").await;
    ui.assert_shows("Catalogue Acryliques (71 références)")
        .await;
    ui.type_into("Rechercher un outil", "séchage").await;
    ui.assert_shows("Temps de Séchage acrylique").await;
    ui.assert_hides("Siccatifs").await;
    ui.press_cmd("k").await;

    ui.click_in("quick-toggles", "EN").await;
    ui.press_cmd("k").await;
    ui.assert_shows("Esc to close").await;
    ui.assert_hides("Échap pour fermer").await;
}
