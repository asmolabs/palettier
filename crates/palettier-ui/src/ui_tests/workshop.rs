//! Projects, layers, palettes and the bench.

use super::harness::Ui;
use dioxus::prelude::*;

async fn new_project(ui: &mut Ui, name: &str, system: &str) {
    ui.click_in("sidebar", "Projets").await;
    ui.type_into("ex. Grognard de la Garde", name).await;
    ui.click_in("system-switch", system).await;
    ui.click("+ Créer le projet").await;
}

#[tokio::test(flavor = "current_thread")]
async fn creating_an_acrylic_project_opens_it_in_acrylic() {
    let mut ui = Ui::studio().await;
    new_project(&mut ui, "Space Marine", "Acrylique").await;

    ui.assert_shows("Mes pièces (1)").await;
    ui.assert_shows("Space Marine 1 zone").await;
    ui.assert_shows("Space Marine Acrylique 7 tubes figés")
        .await;
    // No subject was given: no empty « Sujet : » line.
    ui.assert_hides("Sujet :").await;
    // The sidebar follows the open project.
    ui.assert_shows("Tiroir acrylique").await;
    // Its starting layers use acrylic techniques and acrylic paints only.
    ui.assert_shows("Lavis d'ombrage").await;
    ui.assert_shows("Liserés d'arêtes (edge highlight)").await;
    ui.assert_hides("Jus à l'huile").await;
    let project = ui.peek(|s| s.projects.peek()[0].clone());
    assert!(
        project
            .paints
            .iter()
            .all(|p| p.system == palettier_domain::paint::PaintSystem::Acrylic)
    );
}

#[tokio::test(flavor = "current_thread")]
async fn a_project_needs_a_name() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Projets").await;
    ui.click("+ Créer le projet").await;
    ui.assert_shows("Mes pièces (0)").await;
    assert!(ui.peek(|s| s.projects.peek().is_empty()));
}

#[tokio::test(flavor = "current_thread")]
async fn marking_a_layer_applied_puts_it_on_the_bench() {
    let mut ui = Ui::studio().await;
    new_project(&mut ui, "Grenadier", "Huile").await;
    ui.assert_shows("0/5 posées").await;

    ui.click("Marquer posée").await;
    ui.assert_shows("1/5 posées").await;
    let layer = ui.peek(|s| s.projects.peek()[0].zones[0].layers[0].clone());
    assert!(layer.applied.is_some(), "the first layer is stamped");

    ui.click_in("sidebar", "Aujourd'hui").await;
    ui.assert_shows("Pièces en cours (1)").await;
    ui.assert_shows("Grenadier").await;
}

#[tokio::test(flavor = "current_thread")]
async fn adding_a_zone_adds_five_layers() {
    let mut ui = Ui::studio().await;
    new_project(&mut ui, "Hussard", "Huile").await;
    ui.type_into("Nouvelle zone (ex. Manteau)", "Dolman").await;
    ui.type_into("Matière (ex. Drap de laine)", "Drap").await;
    ui.click("+ Ajouter la zone").await;

    ui.assert_shows("Dolman · Drap").await;
    ui.assert_shows("Hussard 2 zones").await;
    assert_eq!(ui.peek(|s| s.projects.peek()[0].zones.len()), 2);
}

#[tokio::test(flavor = "current_thread")]
async fn deleting_a_project_says_so() {
    let mut ui = Ui::studio().await;
    new_project(&mut ui, "Dragon", "Huile").await;
    ui.click("Supprimer définitivement ce projet").await;

    ui.assert_shows("Projet « Dragon » supprimé.").await;
    ui.assert_shows("Mes pièces (0)").await;
    ui.click_in("toast-banner", "Fermer").await;
    ui.assert_hides("supprimé").await;
}

#[tokio::test(flavor = "current_thread")]
async fn creating_a_palette_lists_it() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Palettes").await;
    ui.assert_shows("Palettes enregistrées (3)").await;
    ui.type_into("ex. Drap impérial bleu", "Drap bleu").await;
    ui.type_into("ex. Uniformes 75mm", "Uniformes").await;
    ui.click("+ Créer la palette").await;

    ui.assert_shows("Palettes enregistrées (4)").await;
    ui.assert_shows("Drap bleu").await;
}

#[tokio::test(flavor = "current_thread")]
async fn the_hud_follows_its_piece_when_the_list_changes() {
    let mut ui = Ui::studio().await;
    new_project(&mut ui, "Hussard", "Huile").await;
    new_project(&mut ui, "Dragon", "Huile").await;
    let hussard = ui.peek(|s| {
        s.projects
            .peek()
            .iter()
            .find(|p| p.name == "Hussard")
            .and_then(|p| p.id)
            .expect("saved")
    });

    ui.press_cmd("h").await;
    ui.assert_shows("HUD Établi Peintre").await;
    ui.type_into("Pièce suivie", &hussard.to_string()).await;

    // Another piece leaves the list: the HUD stays on the Hussard.
    ui.set_state(|s| s.projects.write().retain(|p| p.name != "Dragon"))
        .await;
    ui.set_state(|s| {
        let all = s.projects.peek().clone();
        let mut reordered = all.clone();
        reordered.reverse();
        s.projects.set(reordered);
    })
    .await;
    let select = ui
        .tree()
        .descendants()
        .into_iter()
        .find(|e| e.attr("title") == Some("Pièce suivie"))
        .cloned()
        .expect("piece selector");
    let chosen: Vec<String> = select
        .descendants()
        .into_iter()
        .filter(|e| e.tag == "option" && e.attr("selected") == Some("true"))
        .map(|e| e.text())
        .collect();
    assert_eq!(chosen, vec!["Hussard".to_string()]);
}
