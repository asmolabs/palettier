//! Catalog, drying and recipes.

use super::harness::Ui;

#[tokio::test(flavor = "current_thread")]
async fn catalog_search_narrows_the_list() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Catalogue").await;
    ui.assert_shows("680 tube(s) affiché(s)").await;
    ui.assert_shows("Affichage de 250 sur 680 références").await;
    ui.click("Afficher +250").await;
    ui.assert_shows("Affichage de 500 sur 680 références").await;
    ui.type_into("Rechercher par nom, marque, code ou pigment", "PB29")
        .await;
    ui.assert_hides("680 tube(s) affiché(s)").await;
    ui.assert_shows("Ultramarine").await;
}

#[tokio::test(flavor = "current_thread")]
async fn mixer_solver_runs_asynchronously_on_palette() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Mélangeur").await;
    ui.click("Proposer des recettes").await;
    ui.assert_shows("Dosage proposé").await;
}

#[tokio::test(flavor = "current_thread")]
async fn mixer_bench_updates_brands_when_switching_system() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Mélangeur").await;
    ui.click_in("system-segment", "Acrylique").await;
    ui.click("+ Ajouter").await;
    ui.assert_shows("Vider la paillasse").await;
}

#[tokio::test(flavor = "current_thread")]
async fn mixer_in_acrylic_offers_acrylics_only() {
    let mut ui = Ui::studio().await;
    ui.click_in("system-segment", "Acrylique").await;
    ui.click_in("sidebar", "Mélangeur").await;
    let options: Vec<String> = ui
        .tree()
        .descendants()
        .into_iter()
        .filter(|e| e.tag == "option")
        .map(|e| e.text())
        .collect();
    assert!(
        options.iter().any(|o| o.contains("Citadel")),
        "{options:#?}"
    );
    let oil: Vec<&String> = options
        .iter()
        .filter(|o| o.contains("Abteilung 502") || o.contains("Winsor & Newton"))
        .collect();
    assert!(oil.is_empty(), "oil tubes offered in acrylic: {oil:#?}");
}

#[tokio::test(flavor = "current_thread")]
async fn mixer_picks_acrylic_target_for_oil_mix_and_vice_versa() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Mélangeur").await;

    // In Oil mode, the cross-system target picker defaults to Acrylic: search Mephiston Red.
    ui.type_into("Chercher un pot/tube cible", "Mephiston Red").await;
    ui.click_in("target-paint-grid", "Mephiston Red").await;
    ui.assert_shows("Citadel · Mephiston Red").await;
    ui.assert_shows("(Acrylique → Huile)").await;
    // Selecting a target paint automatically solves for an Oil mix.
    ui.assert_shows("Dosage proposé").await;

    // Switch studio to Acrylic: target picker defaults to Oil; search Burnt Umber.
    ui.click_in("system-segment", "Acrylique").await;
    ui.type_into("Chercher un pot/tube cible", "Burnt Umber").await;
    ui.click_in("target-paint-grid", "Burnt Umber").await;
    ui.assert_shows("(Huile → Acrylique)").await;
    ui.assert_shows("Dosage proposé").await;
}

#[tokio::test(flavor = "current_thread")]
async fn drying_follows_the_thickness_chosen() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Séchage").await;
    let before = ui.text();
    ui.click("Empâtement").await;
    let after = ui.text();
    assert_ne!(before, after, "an impasto dries slower than a normal layer");
    ui.assert_shows("Pâte épaisse").await;
}

#[tokio::test(flavor = "current_thread")]
async fn acrylic_drying_counts_in_minutes() {
    let mut ui = Ui::studio().await;
    ui.click_in("system-segment", "Acrylique").await;
    ui.click_in("sidebar", "Séchage").await;
    ui.assert_shows("Sec au toucher (hors poussière) 15 min")
        .await;
    // Fat over lean is an oil rule: no fat index in acrylic.
    ui.assert_hides("indice gras").await;
    // Acrylic dries by evaporation: no polymerization before the varnish.
    ui.assert_shows("Film sec à cœur (coalescence) avant vernis définitif").await;
    ui.assert_hides("Polymérisation complète").await;
}

#[tokio::test(flavor = "current_thread")]
async fn a_new_recipe_gets_its_steps() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Recettes").await;
    ui.assert_shows("Recettes enregistrées (3)").await;
    ui.type_into("Ex: Carnation chaude en 4 étapes", "Cuir usé")
        .await;
    ui.type_into("Ex: Visage, Cuir usé, Drapé rouge…", "Ceinturon")
        .await;
    ui.click("Créer la recette").await;
    ui.assert_shows("Recettes enregistrées (4)").await;
    ui.assert_shows("Ceinturon · 0 étape(s)").await;
    // The new recipe is the one open on the right.
    ui.assert_shows("Cuir usé Sujet : Ceinturon").await;

    // Adding a step updates the saved recipe (it used to fail on the unique name).
    ui.type_into("Ex: Ocre jaune + Blanc de titane", "Terre d'ombre brûlée")
        .await;
    ui.click("Ajouter l'étape").await;
    ui.assert_shows("Ceinturon · 1 étape(s)").await;
    ui.assert_shows("Recettes enregistrées (4)").await;
    ui.assert_shows("Terre d'ombre brûlée").await;
}

#[tokio::test(flavor = "current_thread")]
async fn deleting_a_recipe_removes_it() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Recettes").await;
    ui.click("Supprimer").await;
    ui.assert_shows("Recettes enregistrées (2)").await;
}

#[tokio::test(flavor = "current_thread")]
async fn catalog_custom_tube_and_sheet_open_in_slide_drawer() {
    let mut ui = Ui::studio().await;
    ui.click_in("sidebar", "Catalogue").await;
    ui.assert_hides("Enregistrer ce tube").await;

    ui.click("+ Nouveau tube personnalisé").await;
    assert!(ui.find_class("drawer-panel").is_some());
    ui.type_into("ex. Ocre d'or maison", "Ocre Atelier Test").await;
    ui.click("Enregistrer ce tube").await;

    // Submitting closes the custom-tube drawer and notifies via the floating toast.
    ui.assert_shows("Ocre Atelier Test").await;
    ui.assert_shows("ajouté au catalogue et au tiroir.").await;
    // The newly created tube is selected in the sheet drawer; closing clears it.
    assert!(ui.find_class("drawer-panel").is_some());
    ui.click_in("drawer-panel", "Fermer (Échap)").await;
    assert!(ui.find_class("drawer-panel").is_none());
}
