//! The open piece: its toolbar, tubes, warnings, zones, progress check and photo journal.

use super::projects::ProjectUi;
use super::zones::ZoneCard;
use crate::actions::projects;
use crate::state::{AppServices, AppState};
use crate::theme::{
    ColorSwatch, DeltaEBadge, MixingGuideCard, PaintSwatch, export_and_open_guide_html,
};
use base64::Engine;
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::PaintSystem;
use palettier_domain::project::{MissingPaint, PhotoRole, Project, ProjectPhoto};
use std::str::FromStr;

/// A photo as an inline `data:` URI.
fn data_uri(photo: &ProjectPhoto) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(&photo.data);
    format!("data:image/jpeg;base64,{b64}")
}

#[component]
pub fn ProjectDetail(proj: Project) -> Element {
    let state = use_context::<AppState>();
    let ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "card",
            ProjectToolbar { proj: proj.clone() }

            if (ui.show_guide)() {
                ProjectGuide { proj: proj.clone() }
            }

            // Locked project paints banner + Sync button if palette changed
            if !proj.effective_paints().is_empty() {
                ProjectTubes { proj: proj.clone() }
            }

            // Fat-over-lean & missing tube substitutes
            ProjectWarnings { proj: proj.clone() }

            // Zones and layers with ValueLadderStrip + live Kubelka-Munk recipe + inline edit
            for (z_idx, _) in proj.zones.iter().enumerate() {
                ZoneCard { proj: proj.clone(), z_idx }
            }

            AddZoneRow { proj: proj.clone() }

            // Progress check observations
            if !ui.progress_observations.read().is_empty() {
                ProgressCard {}
            }

            // Photo journal with thumbnails
            if !proj.photos.is_empty() {
                div { style: "margin-top:16px;",
                    h3 { class: "card-title", {locale.tr("Journal photo", "Photo Journal")}, " ({proj.photos.len()})" }
                    div { class: "photo-grid",
                        for ph in proj.photos.clone() {
                            PhotoCard { proj: proj.clone(), ph }
                        }
                    }
                }
            }
        }
    }
}

/// Name and system of the piece, with the guide, export, photo and delete buttons.
#[component]
fn ProjectToolbar(proj: Project) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();
    let show_guide = (ui.show_guide)();

    let export_guide = {
        let svc = services.clone();
        let p_export = proj.clone();
        move |_| {
            let loc = *state.locale.read();
            let guide = svc.guide_builder.from_project_i18n(&p_export, loc);
            let photo_uri = projects::showcase_photo(&p_export).map(data_uri);
            match export_and_open_guide_html(&guide, photo_uri.as_deref(), loc) {
                Ok(path) => {
                    let msg = match loc {
                        Locale::Fr => format!("Guide de mélanges A4 ouvert : {}", path.display()),
                        Locale::En => format!("A4 Mixing Guide opened: {}", path.display()),
                    };
                    state.notify(msg);
                }
                Err(e) => state.notify(e),
            }
        }
    };

    let add_photo = {
        let svc = services.clone();
        let p_for_check = proj.clone();
        move |_| {
            let svc = svc.clone();
            let p_for_check = p_for_check.clone();
            let role = *ui.photo_role_to_add.read();
            spawn(async move {
                let loc = *state.locale.read();
                let Some(id) = p_for_check.id else { return };
                let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                    .pick_file()
                    .await
                else {
                    return;
                };
                let Ok(bytes) = std::fs::read(file.path()) else { return };
                if role == PhotoRole::Progress
                    && let Ok(obs) = projects::compare_to_plan(&svc, &p_for_check, &bytes)
                {
                    ui.progress_observations.set(obs);
                }
                if let Ok(updated) = projects::add_photo(&svc, id, bytes, role, file.file_name()).await {
                    ui.full_project.set(Some(updated));
                    state.notify(loc.tr("Photo ajoutée au journal de la pièce.", "Photo added to piece journal."));
                }
            });
        }
    };

    let export_card = {
        let svc = services.clone();
        let p_card = proj.clone();
        move |_| {
            let loc = *state.locale.read();
            let photo_bytes = projects::showcase_photo(&p_card).map(|ph| ph.data.as_slice());
            let swatches: Vec<String> = p_card
                .effective_paints()
                .iter()
                .map(|p| p.hex_color.clone())
                .collect();
            if let Ok(card_png) = svc.decoder.render_social_recipe_card(photo_bytes, &swatches)
                && let Some(dest) = rfd::FileDialog::new()
                    .add_filter("PNG Image", &["png"])
                    .set_file_name(format!("{}-recipe-card.png", p_card.name))
                    .save_file()
                && std::fs::write(&dest, card_png).is_ok()
            {
                state.notify(loc.tr(
                    "Carte recette PNG exportée avec succès !",
                    "Recipe card PNG exported successfully!",
                ));
            }
        }
    };

    let delete = {
        let pid = proj.id;
        move |_| {
            let svc = services.clone();
            spawn(async move {
                if let Some(id) = pid
                    && let Ok(all) = projects::delete_project(&svc, id).await
                {
                    ui.selected_id.set(all.first().and_then(|p| p.id));
                    state.projects.set(all);
                }
            });
        }
    };

    rsx! {
        div { class: "row-between", style: "margin-bottom:12px;",
            div {
                h2 { class: "card-title", style: "font-size:19px;margin-bottom:2px;",
                    "{proj.name} "
                    span {
                        class: if proj.system == PaintSystem::Acrylic { "pill pill-acrylic" } else { "pill pill-oil" },
                        title: "{proj.system.summary_i18n(locale)}",
                        "{proj.system.label_i18n(locale)}"
                    }
                }
                span { class: "muted",
                    "{locale.tr(\"Sujet\", \"Subject\")} : {proj.subject} · {proj.effective_paints().len()} ",
                    {locale.tr("tubes figés", "locked tubes")}
                }
            }
            div { class: "row project-toolbar",
                button {
                    class: if show_guide { "btn-primary" } else { "btn" },
                    onclick: move |_| ui.show_guide.set(!show_guide),
                    if show_guide {
                        {locale.tr("← Retour aux couches", "← Back to layers")}
                    } else {
                        {locale.tr("Guide de mélanges (Norma Blue)", "Mixing Guide (Norma Blue)")}
                    }
                }
                button {
                    class: "btn",
                    onclick: export_guide,
                    {locale.tr("Imprimer / Exporter Guide A4", "Print / Export A4 Guide")}
                }
                span { class: "toolbar-sep" }
                select {
                    value: "{(ui.photo_role_to_add)().as_db_str()}",
                    onchange: move |e| {
                        if let Ok(r) = PhotoRole::from_str(&e.value()) {
                            ui.photo_role_to_add.set(r);
                        }
                    },
                    for r in PhotoRole::ALL {
                        option { value: "{r.as_db_str()}", "Photo: {r.label_i18n(locale)}" }
                    }
                }
                button {
                    class: "btn",
                    onclick: add_photo,
                    {locale.tr("+ Ajouter photo", "+ Add photo")}
                }
                button {
                    class: "btn",
                    onclick: export_card,
                    "📷 "
                    {locale.tr("Carte PNG", "PNG Card")}
                }
                span { class: "toolbar-spacer" }
                button {
                    class: "btn-danger",
                    onclick: delete,
                    {locale.tr("Supprimer", "Delete")}
                }
            }
        }
    }
}

/// The printable mixing guide of the piece, shown in place of the layers.
#[component]
fn ProjectGuide(proj: Project) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let guide = services.guide_builder.from_project_i18n(&proj, locale);
    let photo_uri = projects::showcase_photo(&proj).map(data_uri);

    rsx! {
        div { style: "margin-bottom: 18px;",
            MixingGuideCard {
                guide: guide,
                photo_data_uri: photo_uri,
                on_notify: move |msg: String| state.notify(msg),
            }
        }
    }
}

/// The tubes locked in the piece, with a button to take the palette's current tubes.
#[component]
fn ProjectTubes(proj: Project) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();

    let sync = {
        let p_sync = proj.clone();
        move |_| {
            let svc = services.clone();
            let p_sync = p_sync.clone();
            spawn(async move {
                let loc = *state.locale.read();
                if let Ok(change) = projects::sync_project_paints(&svc, &p_sync).await {
                    ui.show_reloaded(state, &svc, change).await;
                    state.notify(loc.tr("Palette du projet synchronisée avec la composition actuelle.", "Project palette synced with current composition."));
                }
            });
        }
    };

    rsx! {
        div { class: "card-subtle", style: "margin-bottom: 14px;",
            div { class: "row-between", style: "margin-bottom: 8px;",
                span { class: "field-label", style: "margin: 0;",
                    {locale.tr("Tubes du projet", "Project Tubes")}, " ({proj.effective_paints().len()})"
                }
                if proj.diverges_from_palette() {
                    button {
                        class: "btn-primary",
                        onclick: sync,
                        {locale.tr("Reprendre la palette actuelle", "Sync with current palette")}
                    }
                }
            }
            div { class: "row",
                for pt in proj.effective_paints() {
                    div { class: "pill", style: "gap: 6px; padding: 3px 9px;",
                        PaintSwatch {
                            masstone_hex: pt.hex_color.clone(),
                            tint_hex: pt.tint_hex.clone(),
                            width: 22,
                            height: 14,
                        }
                        span { "{pt.name}" }
                        if !pt.in_stock {
                            span { style: "color: var(--danger); font-size: 10px;",
                                " (", {locale.tr("hors stock", "out of stock")}, ")"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Fat-over-lean (and acrylic-over-oil) warnings, and out-of-stock tubes with their substitutes.
#[component]
fn ProjectWarnings(proj: Project) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();

    let risks = services.fat_over_lean.inspect_project_i18n(&proj, locale);
    let owned_paints: Vec<_> = state
        .paints
        .read()
        .iter()
        .filter(|p| p.in_stock)
        .cloned()
        .collect();
    let missing = services.substitute_service.missing_from(&proj, &owned_paints);

    rsx! {
        if !risks.is_empty() {
            div { class: "card-subtle", style: "border-color:var(--pill-warn-border);margin-bottom:14px;",
                strong { style: "color:var(--pill-warn-text);",
                    {locale.tr("Avertissements Gras sur Maigre", "Fat Over Lean Warnings")}, " ({risks.len()})"
                }
                for r in risks {
                    div { class: "muted", style: "font-size:12px;margin-top:4px;", "• {r.where_name} ({r.under} → {r.over}) : {r.explanation}" }
                }
            }
        }
        if !missing.is_empty() {
            div { class: "card-subtle", style: "border-color:var(--accent);margin-bottom:14px;",
                strong {
                    {locale.tr("Tubes hors stock et substituts sur votre étagère", "Out-of-stock tubes & substitutes in your drawer")}, " ({missing.len()})"
                }
                for m in missing {
                    ProjectSubstituteRow { proj: proj.clone(), m }
                }
            }
        }
    }
}

#[component]
fn ProjectSubstituteRow(proj: Project, m: MissingPaint) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();
    let absent_id = m.paint.id;

    let replace = move |replacement: palettier_domain::paint::Paint| {
        let svc = services.clone();
        let proj = proj.clone();
        spawn(async move {
            let loc = *state.locale.read();
            if let Ok(change) = projects::substitute_project_paint(&svc, &proj, absent_id, replacement).await {
                ui.show_reloaded(state, &svc, change).await;
                state.notify(loc.tr("Tube hors stock remplacé par son équivalent en stock.", "Out-of-stock tube replaced with in-stock substitute."));
            }
        });
    };

    rsx! {
        div { class: "row-between", style: "margin-top:8px; padding-top:6px; border-top:1px solid var(--border-subtle);",
            div { class: "row",
                ColorSwatch { hex: m.paint.hex_color.clone(), size: 20 }
                strong { "{m.paint.display_name()}" }
                span { class: "muted", "→" }
                if let Some(ref near) = m.nearest {
                    ColorSwatch { hex: near.hex_color.clone(), size: 20 }
                    DeltaEBadge { delta_e: m.delta_e }
                }
                span { class: "muted", style: "font-size:12px;", "{m.verdict_i18n(locale)}" }
            }
            if let Some(replacement) = m.nearest.clone() {
                button {
                    class: "btn-primary",
                    style: "padding: 3px 10px; font-size: 11.5px;",
                    onclick: move |_| replace(replacement.clone()),
                    {locale.tr("Remplacer dans le projet", "Replace in project")}
                }
            }
        }
    }
}

/// Name and material of a new zone.
#[component]
fn AddZoneRow(proj: Project) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();

    let add = move |_| {
        let loc = *state.locale.read();
        let zname = (ui.new_zone_name)();
        if zname.trim().is_empty() {
            return;
        }
        let zmat = (ui.new_zone_mat)();
        let svc = services.clone();
        let proj = proj.clone();
        spawn(async move {
            if let Ok(Some(change)) = projects::add_zone(&svc, &proj, &zname, &zmat, loc).await {
                ui.show(state, change);
                ui.new_zone_name.set(String::new());
                ui.new_zone_mat.set(String::new());
            }
        });
    };

    rsx! {
        div { class: "row", style: "margin-top:10px;",
            input {
                r#type: "text",
                placeholder: locale.tr("Nouvelle zone (ex. Manteau)", "New zone (e.g. Coat)"),
                value: "{ui.new_zone_name}",
                oninput: move |ev| ui.new_zone_name.set(ev.value())
            }
            input {
                r#type: "text",
                placeholder: locale.tr("Matière (ex. Drap de laine)", "Material (e.g. Wool cloth)"),
                value: "{ui.new_zone_mat}",
                oninput: move |ev| ui.new_zone_mat.set(ev.value())
            }
            button {
                class: "btn",
                onclick: add,
                {locale.tr("+ Ajouter la zone", "+ Add zone")}
            }
        }
    }
}

/// The colours measured on a progress photo, against the planned coats.
#[component]
fn ProgressCard() -> Element {
    let state = use_context::<AppState>();
    let mut ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "card-subtle", style: "margin-top: 16px; border-color: var(--accent);",
            div { class: "row-between", style: "margin-bottom: 8px;",
                strong { {locale.tr("Comparaison d'avancement sur la photo relevée", "Progress comparison on sampled photo")} }
                button {
                    class: "btn",
                    onclick: move |_| ui.progress_observations.set(Vec::new()),
                    {locale.tr("Fermer", "Close")}
                }
            }
            for obs in ui.progress_observations.read().iter() {
                div { class: "row-between", style: "padding: 5px 0; border-top: 1px solid var(--border-subtle);",
                    div { class: "row",
                        ColorSwatch { hex: obs.measured.to_hex(), size: 22 }
                        if let Some(target) = obs.target {
                            span { class: "muted", "vs" }
                            ColorSwatch { hex: target.to_hex(), size: 22 }
                        }
                        span { style: "font-size: 12.5px;", "{obs.verdict_i18n(locale)}" }
                    }
                    if obs.matches_plan() {
                        DeltaEBadge { delta_e: obs.delta_e }
                    }
                }
            }
        }
    }
}

/// One photo of the journal, with « compare to plan » and remove.
#[component]
fn PhotoCard(proj: Project, ph: ProjectPhoto) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();
    let src = data_uri(&ph);

    let compare = {
        let svc = services.clone();
        let ph_data = ph.data.clone();
        let proj = proj.clone();
        move |_| {
            if let Ok(obs) = projects::compare_to_plan(&svc, &proj, &ph_data) {
                ui.progress_observations.set(obs);
            }
        }
    };

    let remove = move |(pid, fid): (i64, i64)| {
        let svc = services.clone();
        spawn(async move {
            if let Ok(updated) = projects::remove_photo(&svc, pid, fid).await {
                ui.full_project.set(Some(updated));
            }
        });
    };

    rsx! {
        div { class: "card-subtle",
            img {
                class: "photo-thumb",
                src: "{src}",
                alt: "{ph.caption}",
            }
            div { class: "row-between",
                strong { style: "font-size: 12px; overflow: hidden; text-overflow: ellipsis;", "{ph.caption}" }
                span { class: "pill", "{ph.role.label_i18n(locale)}" }
            }
            div { class: "row", style: "margin-top: 6px;",
                button {
                    class: "btn",
                    style: "padding: 3px 8px; font-size: 11px;",
                    onclick: compare,
                    {locale.tr("Comparer au plan", "Compare to plan")}
                }
                if let (Some(pid), Some(fid)) = (proj.id, ph.id) {
                    button {
                        class: "btn-danger",
                        style: "padding: 3px 8px; font-size: 11px;",
                        onclick: move |_| remove((pid, fid)),
                        "✕"
                    }
                }
            }
        }
    }
}
