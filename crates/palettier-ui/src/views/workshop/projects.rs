//! Projects & pieces: the list of pieces, the new-piece card, and the open piece.

use super::project_detail::ProjectDetail;
use crate::actions::projects::{self, LayerEdit, NewProject, ProjectChange};
use crate::state::{AppServices, AppState};
use crate::theme::SectionHeader;
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::{PaintSystem, Technique};
use palettier_domain::palette::Palette;
use palettier_domain::project::{ObservedColour, PhotoRole, Project};

/// The state of the Projects screen shared by its cards (given as context).
#[derive(Clone, Copy)]
pub struct ProjectUi {
    pub selected_id: Signal<Option<i64>>,
    /// The open piece, read with its photos.
    pub full_project: Signal<Option<Project>>,
    pub progress_observations: Signal<Vec<ObservedColour>>,
    /// The coat being edited inline: (zone index, layer index).
    pub editing_layer: Signal<Option<(usize, usize)>>,
    pub edit_role: Signal<String>,
    pub edit_hex: Signal<String>,
    pub edit_tech: Signal<Technique>,
    pub edit_note: Signal<String>,
    pub show_guide: Signal<bool>,
    pub photo_role_to_add: Signal<PhotoRole>,
    pub new_zone_name: Signal<String>,
    pub new_zone_mat: Signal<String>,
}

impl ProjectUi {
    /// Shows a piece after a change: the lists and the open piece.
    pub fn show(&mut self, mut state: AppState, change: ProjectChange) {
        state.projects.set(change.projects);
        if let Some(project) = change.project {
            self.full_project.set(Some(project));
        }
    }

    /// Shows a piece after a change that may touch the palettes and the catalog too:
    /// everything is reloaded, then the open piece.
    pub async fn show_reloaded(&mut self, mut state: AppState, services: &AppServices, change: ProjectChange) {
        state.reload_all(services).await;
        if let Some(project) = change.project {
            self.full_project.set(Some(project));
        }
    }

    /// The fields of the inline coat editor.
    pub fn layer_edit(&self) -> LayerEdit {
        LayerEdit {
            role: (self.edit_role)(),
            target_hex: (self.edit_hex)(),
            technique: (self.edit_tech)(),
            note: (self.edit_note)(),
        }
    }
}

#[component]
pub fn ProjectsView() -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();

    let locale = *state.locale.read();
    let projects = state.projects.read().clone();
    let first_id = projects.first().and_then(|p| p.id);
    let mut ui = use_context_provider(|| ProjectUi {
        selected_id: Signal::new(first_id),
        full_project: Signal::new(None),
        progress_observations: Signal::new(Vec::new()),
        editing_layer: Signal::new(None),
        edit_role: Signal::new(String::new()),
        edit_hex: Signal::new(String::new()),
        edit_tech: Signal::new(Technique::Glaze),
        edit_note: Signal::new(String::new()),
        show_guide: Signal::new(false),
        photo_role_to_add: Signal::new(PhotoRole::Progress),
        new_zone_name: Signal::new(String::new()),
        new_zone_mat: Signal::new(String::new()),
    });

    // Load the selected project along with its photos
    let current_id = (ui.selected_id)();
    let svc_for_effect = services.clone();
    use_effect(use_reactive!(|current_id| {
        let svc = svc_for_effect.clone();
        spawn(async move {
            if let Some(id) = current_id {
                if let Ok(p) = projects::load_project(&svc, id).await {
                    ui.full_project.set(p);
                }
            } else {
                ui.full_project.set(None);
            }
            ui.progress_observations.set(Vec::new());
            ui.editing_layer.set(None);
        });
    }));

    let active_proj = (ui.full_project)()
        .or_else(|| projects.iter().find(|p| p.id == (ui.selected_id)()).cloned());

    // The open piece sets the working system of the other screens.
    let active_system = active_proj.as_ref().map(|p| p.system);
    let svc_for_system = services.clone();
    use_effect(use_reactive!(|active_system| {
        if let Some(sys) = active_system {
            state.set_paint_system(&svc_for_system, sys);
        }
    }));

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Projets & Pièces", "Projects & Pieces").to_string(),
                subtitle: locale.tr(
                    "Échelles de valeurs par zone, recettes Kubelka-Munk en direct, suivi de pose, journal photo et contrôle d'avancement.",
                    "Per-zone value ladders, live Kubelka-Munk recipes, coat tracking, photo journal, and progress check."
                ).to_string(),
                actions: None
            }

            div { class: "grid-3",
                // Column 1: Project list + creation
                div {
                    NewProjectCard {}
                    ProjectList {}
                }

                // Columns 2 & 3: Active project details
                div { style: "grid-column: span 2;",
                    if let Some(proj) = active_proj {
                        ProjectDetail { proj }
                    } else {
                        div { class: "card",
                            p { class: "muted", {locale.tr("Sélectionnez ou créez un projet dans la colonne de gauche.", "Select or create a project in the left column.")} }
                        }
                    }
                }
            }
        }
    }
}

/// Name, subject, system and palette of a new piece.
#[component]
fn NewProjectCard() -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();

    let mut new_proj_system = use_signal(|| *state.paint_system.peek());
    // A new piece only offers the palettes of its own system.
    let palettes: Vec<Palette> = state
        .palettes
        .read()
        .iter()
        .filter(|p| p.paints.is_empty() || p.system() == new_proj_system())
        .cloned()
        .collect();
    let mut new_proj_name = use_signal(String::new);
    let mut new_proj_subject = use_signal(String::new);
    let mut new_proj_palette_idx = use_signal(|| 0usize);

    let create = {
        let pals = palettes.clone();
        move |_| {
            let loc = *state.locale.read();
            let draft = NewProject {
                name: new_proj_name(),
                subject: new_proj_subject(),
                system: new_proj_system(),
                palette: pals.get(new_proj_palette_idx()).cloned(),
            };
            if draft.name.trim().is_empty() {
                return;
            }
            let svc = services.clone();
            spawn(async move {
                if let Ok(Some(change)) = projects::create_project(&svc, draft, loc).await {
                    ui.selected_id.set(change.project.as_ref().and_then(|p| p.id));
                    ui.show(state, change);
                    new_proj_name.set(String::new());
                    new_proj_subject.set(String::new());
                }
            });
        }
    };

    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("Nouveau projet", "New Project")} }
            div { style: "margin-bottom:8px;",
                label { class: "field-label", {locale.tr("Nom de la pièce", "Piece Name")} }
                input {
                    r#type: "text",
                    style: "width:100%;",
                    placeholder: locale.tr("ex. Grognard de la Garde", "e.g. Old Guard Grenadier"),
                    value: "{new_proj_name}",
                    oninput: move |ev| new_proj_name.set(ev.value())
                }
            }
            div { style: "margin-bottom:8px;",
                label { class: "field-label", {locale.tr("Sujet / Échelle", "Subject / Scale")} }
                input {
                    r#type: "text",
                    style: "width:100%;",
                    placeholder: locale.tr("ex. Buste 1/10", "e.g. 1/10 Bust"),
                    value: "{new_proj_subject}",
                    oninput: move |ev| new_proj_subject.set(ev.value())
                }
            }
            div { style: "margin-bottom:8px;",
                label { class: "field-label", {locale.tr("Peinte à", "Painted in")} }
                div { class: "system-switch",
                    for sys in PaintSystem::ALL {
                        button {
                            class: if new_proj_system() == sys { "toggle-chip active" } else { "toggle-chip" },
                            onclick: move |_| {
                                new_proj_system.set(sys);
                                new_proj_palette_idx.set(0);
                            },
                            "{sys.label(locale)}"
                        }
                    }
                }
                p { class: "muted", style: "font-size:11px; margin-top:4px;",
                    "{new_proj_system().summary_i18n(locale)}"
                }
            }
            div { style: "margin-bottom:10px;",
                label { class: "field-label", {locale.tr("Palette associée", "Linked Palette")} }
                select {
                    style: "width:100%;",
                    onchange: move |ev| {
                        if let Ok(idx) = ev.value().parse::<usize>() {
                            new_proj_palette_idx.set(idx);
                        }
                    },
                    for (idx, pal) in palettes.iter().enumerate() {
                        option { value: "{idx}", "{pal.name} ({pal.paints.len()} tubes)" }
                    }
                }
            }
            button {
                class: "btn-primary",
                style: "width:100%;",
                onclick: create,
                {locale.tr("+ Créer le projet", "+ Create project")}
            }
        }
    }
}

/// « Mes pièces »: every piece, the open one highlighted.
#[component]
fn ProjectList() -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let projects = state.projects.read().clone();

    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("Mes pièces", "My Pieces")}, " ({projects.len()})" }
            for p in projects {
                ProjectListItem { p }
            }
        }
    }
}

#[component]
fn ProjectListItem(p: Project) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();
    let pid = p.id;

    let delete = {
        let pname = p.name.clone();
        move |ev: Event<MouseData>| {
            ev.stop_propagation();
            let svc = services.clone();
            let pname = pname.clone();
            spawn(async move {
                if let Some(id) = pid
                    && let Ok(all) = projects::delete_project(&svc, id).await
                {
                    ui.selected_id.set(all.first().and_then(|pr| pr.id));
                    state.projects.set(all);
                    let msg = match *state.locale.peek() {
                        Locale::Fr => format!("Projet « {pname} » supprimé."),
                        Locale::En => format!("Project “{pname}” deleted."),
                    };
                    state.notify(msg);
                }
            });
        }
    };

    rsx! {
        div {
            class: "card-subtle",
            style: if pid == (ui.selected_id)() {
                "margin-bottom:8px;border-color:var(--accent);cursor:pointer;"
            } else {
                "margin-bottom:8px;cursor:pointer;"
            },
            onclick: move |_| ui.selected_id.set(pid),
            div { class: "row-between", style: "align-items: center;",
                div {
                    strong { "{p.name}" }
                    if !p.subject.is_empty() {
                        div { class: "muted", style: "font-size:12px;", "{p.subject}" }
                    }
                }
                div { class: "row", style: "gap: 6px; align-items: center;",
                    span { class: "pill", "{p.zones.len()} zones" }
                    button {
                        class: "btn-danger",
                        style: "padding: 3px 7px; font-size: 11px;",
                        title: locale.tr("Supprimer définitivement ce projet", "Delete this project permanently"),
                        onclick: delete,
                        "🗑️"
                    }
                }
            }
        }
    }
}
