//! Global CSS stylesheet (supporting Night/Dark and Day/Light studio themes) and reusable UI components.

use crate::icons::IconClose;
use crate::state::{AppState, ThemeMode};
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;

pub const APP_CSS: &str = include_str!("theme.css");

#[component]
pub fn SectionHeader(title: String, subtitle: String, actions: Option<Element>) -> Element {
    rsx! {
        div { class: "section-header",
            div {
                h1 { class: "page-title", "{title}" }
                p { class: "page-subtitle", "{subtitle}" }
            }
            if let Some(act) = actions {
                div { class: "row", {act} }
            }
        }
    }
}

/// Slide-over right detail/form drawer with blurred backdrop and Escape-to-close support.
#[component]
pub fn SlideDrawer(
    title: String,
    subtitle: Option<String>,
    on_close: EventHandler<()>,
    children: Element,
) -> Element {
    let locale = *use_context::<AppState>().locale.read();
    rsx! {
        div {
            class: "drawer-backdrop",
            tabindex: "0",
            onclick: move |_| on_close.call(()),
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    on_close.call(());
                }
            },
            div {
                class: "drawer-panel",
                onclick: move |e| e.stop_propagation(),
                div { class: "drawer-header",
                    div { class: "drawer-header-text",
                        h3 { class: "drawer-title", "{title}" }
                        if let Some(sub) = subtitle {
                            p { class: "drawer-subtitle", "{sub}" }
                        }
                    }
                    button {
                        class: "icon-btn drawer-close-btn",
                        title: locale.tr("Fermer (Échap)", "Close (Esc)"),
                        onclick: move |_| on_close.call(()),
                        IconClose {}
                    }
                }
                div { class: "drawer-body",
                    {children}
                }
            }
        }
    }
}

#[component]
pub fn ColorSwatch(hex: String, #[props(default = 28)] size: u32) -> Element {
    let style = format!("width: {size}px; height: {size}px; background-color: {hex};");
    rsx! {
        span {
            class: "swatch",
            style: "{style}",
            title: "{hex}",
        }
    }
}

/// Split swatch displaying masstone on the left and 1:10 white tint on the right.
#[component]
pub fn PaintSwatch(
    masstone_hex: String,
    tint_hex: Option<String>,
    #[props(default = 34)] width: u32,
    #[props(default = 24)] height: u32,
) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    if let Some(ref tint) = tint_hex {
        let half = width / 2;
        let tip = match locale {
            Locale::Fr => format!("Sortie de tube {masstone_hex} · Coupe 1:10 {tint}"),
            Locale::En => format!("Masstone {masstone_hex} · 1:10 Tint {tint}"),
        };
        rsx! {
            span {
                class: "split-swatch",
                style: "width: {width}px; height: {height}px;",
                title: "{tip}",
                span {
                    style: "width: {half}px; height: 100%; background-color: {masstone_hex};",
                }
                span {
                    style: "width: {width - half}px; height: 100%; background-color: {tint};",
                }
            }
        }
    } else {
        let style =
            format!("width: {width}px; height: {height}px; background-color: {masstone_hex};");
        rsx! {
            span {
                class: "swatch",
                style: "{style}",
                title: "{masstone_hex}",
            }
        }
    }
}

/// Horizontal value ladder strip for a zone.
#[component]
pub fn ValueLadderStrip(steps: Vec<(String, String)>) -> Element {
    if steps.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "ladder-strip",
            for (role, hex) in steps {
                {
                    let text_col = contrasting_text(&hex);
                    rsx! {
                        div {
                            class: "ladder-step",
                            style: "background-color: {hex}; color: {text_col};",
                            title: "{role} ({hex})",
                            span { "{role}" }
                            span { class: "mono u-fs-9p5 u-op-0p85", "{hex}" }
                        }
                    }
                }
            }
        }
    }
}

/// Color picker combining a native `<input type="color">` and a `#RRGGBB` text input.
#[component]
pub fn HexColorInput(value: String, onchange: EventHandler<String>) -> Element {
    let picker_val = if value.starts_with('#') && value.len() == 7 {
        value.clone()
    } else {
        "#808080".to_string()
    };
    rsx! {
        div { class: "row",
            input {
                r#type: "color",
                value: "{picker_val}",
                oninput: move |e| onchange.call(e.value().to_uppercase()),
            }
            input {
                r#type: "text",
                class: "mono",
                class: "u-w-98",
                value: "{value}",
                oninput: move |e| onchange.call(e.value()),
            }
        }
    }
}

pub fn contrasting_text(hex: &str) -> &'static str {
    if let Ok(rgb) = palettier_domain::color::Rgb::of_hex(hex) {
        if rgb.relative_luminance() > 0.34 {
            "#181410"
        } else {
            "#f5f0ea"
        }
    } else {
        "#f5f0ea"
    }
}

#[component]
pub fn DeltaEBadge(delta_e: f64) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    let (cls, label) = if delta_e < 1.0 {
        (
            "pill pill-ok",
            format!(
                "ΔE {:.1} · {}",
                delta_e,
                locale.tr("imperceptible", "imperceptible")
            ),
        )
    } else if delta_e < 2.0 {
        (
            "pill pill-ok",
            format!(
                "ΔE {:.1} · {}",
                delta_e,
                locale.tr("identique à l'œil", "indistinguishable")
            ),
        )
    } else if delta_e < 4.5 {
        (
            "pill pill-accent",
            format!(
                "ΔE {:.1} · {}",
                delta_e,
                locale.tr("très proche", "very close")
            ),
        )
    } else if delta_e < 8.0 {
        (
            "pill pill-warn",
            format!(
                "ΔE {:.1} · {}",
                delta_e,
                locale.tr("écart visible", "visible gap")
            ),
        )
    } else {
        (
            "pill pill-danger",
            format!("ΔE {:.1} · {}", delta_e, locale.tr("éloigné", "distant")),
        )
    };
    rsx! {
        span { class: "{cls}", "{label}" }
    }
}

/// Computes the 2D convex hull (monotone chain algorithm) of a point cloud `(x, y)`.
fn convex_hull_2d(mut pts: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    pts.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.total_cmp(&b.1)));
    pts.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4);
    if pts.len() <= 2 {
        return pts;
    }
    let cross = |o: (f64, f64), a: (f64, f64), b: (f64, f64)| -> f64 {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    };
    let mut lower = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

fn point_in_convex_polygon(pt: (f64, f64), poly: &[(f64, f64)]) -> bool {
    if poly.len() < 3 {
        return false;
    }
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        let cross = (b.0 - a.0) * (pt.1 - a.1) - (b.1 - a.1) * (pt.0 - a.0);
        if cross < -1.5 {
            return false;
        }
    }
    true
}

/// Interactive CIELAB (`a*` / `b*`) chromaticity gamut map in SVG.
/// Displays the convex hull polygon of the palette, tubes with their short mnemonic codes,
/// as well as the target reticle and achieved mix.
#[component]
pub fn GamutMapSvg(
    paints: Vec<palettier_domain::paint::Paint>,
    target_hex: Option<String>,
    achieved_hex: Option<String>,
    #[props(default = 240)] size: u32,
) -> Element {
    use palettier_domain::color::Rgb;
    use palettier_domain::guide;

    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let theme = *state.theme.read();

    let half = (size as f64) / 2.0;
    let scale = (half - 16.0) / 95.0;
    let to_svg = |a: f64, b: f64| -> (f64, f64) {
        let x = (half + a * scale).clamp(8.0, size as f64 - 8.0);
        let y = (half - b * scale).clamp(8.0, size as f64 - 8.0);
        (x, y)
    };

    let entries = guide::build_palette_entries(&paints, locale);
    let mut lab_pts: Vec<(f64, f64)> = Vec::new();
    let mut tube_markers: Vec<(f64, f64, String, String, String)> = Vec::new();

    for entry in &entries {
        let rgb = entry.paint.color();
        let lab = rgb.to_lab();
        lab_pts.push((lab.a, lab.b));
        if let Some(ref th) = entry.paint.tint_hex
            && let Ok(t_rgb) = Rgb::of_hex(th)
        {
            let t_lab = t_rgb.to_lab();
            lab_pts.push((t_lab.a, t_lab.b));
        }
        let (sx, sy) = to_svg(lab.a, lab.b);
        tube_markers.push((
            sx,
            sy,
            entry.short_code.clone(),
            entry.paint.hex_color.clone(),
            entry.paint.name.clone(),
        ));
    }

    let hull_ab = convex_hull_2d(lab_pts);
    let polygon_points: String = hull_ab
        .iter()
        .map(|&(a, b)| {
            let (x, y) = to_svg(a, b);
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");

    let target_pt = target_hex
        .as_deref()
        .and_then(|h| Rgb::of_hex(h).ok())
        .map(|rgb| {
            let lab = rgb.to_lab();
            let (sx, sy) = to_svg(lab.a, lab.b);
            let inside = point_in_convex_polygon((lab.a, lab.b), &hull_ab);
            (sx, sy, rgb.to_hex(), inside)
        });

    let achieved_pt = achieved_hex
        .as_deref()
        .and_then(|h| Rgb::of_hex(h).ok())
        .map(|rgb| {
            let lab = rgb.to_lab();
            let (sx, sy) = to_svg(lab.a, lab.b);
            (sx, sy, rgb.to_hex())
        });

    let r25 = 25.0 * scale;
    let r50 = 50.0 * scale;
    let r75 = 75.0 * scale;

    let (svg_bg, ring_stroke, axis_stroke, code_fill) = match theme {
        ThemeMode::Dark => (
            "background: radial-gradient(circle at center, #201d1a 0%, #141210 100%); border-radius: 10px; border: 1px solid #2b2724;",
            "#2b2724",
            "#38322c",
            "#e6dfd7",
        ),
        ThemeMode::Light => (
            "background: radial-gradient(circle at center, #faf7f2 0%, #ede6d8 100%); border-radius: 10px; border: 1px solid #d5ccbc;",
            "#d8cfc0",
            "#bfb4a2",
            "#262320",
        ),
        ThemeMode::Neutral => (
            "background: radial-gradient(circle at center, #333333 0%, #262626 100%); border-radius: 10px; border: 1px solid #444444;",
            "#444444",
            "#555555",
            "#ebebeb",
        ),
    };

    let lbl_yellow = locale.tr("+b* Jaune", "+b* Yellow");
    let lbl_blue = locale.tr("-b* Bleu", "-b* Blue");
    let lbl_red = locale.tr("+a* Rouge", "+a* Red");
    let lbl_green = locale.tr("-a* Vert", "-a* Green");

    rsx! {
        div { class: "u-d-flex u-fd-column u-ai-center u-gap-6",
            svg {
                width: "{size}",
                height: "{size}",
                view_box: "0 0 {size} {size}",
                style: "{svg_bg}",

                // Chroma rings C* = 25, 50, 75
                circle { cx: "{half}", cy: "{half}", r: "{r25}", fill: "none", stroke: "{ring_stroke}", stroke_dasharray: "2,2" }
                circle { cx: "{half}", cy: "{half}", r: "{r50}", fill: "none", stroke: "{ring_stroke}", stroke_dasharray: "3,3" }
                circle { cx: "{half}", cy: "{half}", r: "{r75}", fill: "none", stroke: "{ring_stroke}" }

                // Axes a* (Green <-> Red) and b* (Blue <-> Yellow)
                line { x1: "8", y1: "{half}", x2: "{size - 8}", y2: "{half}", stroke: "{axis_stroke}", stroke_width: "1" }
                line { x1: "{half}", y1: "8", x2: "{half}", y2: "{size - 8}", stroke: "{axis_stroke}", stroke_width: "1" }

                // Chromatic pole labels
                text { x: "{half}", y: "14", fill: "#b88a1e", font_size: "9", text_anchor: "middle", "{lbl_yellow}" }
                text { x: "{half}", y: "{size - 6}", fill: "#4a78a8", font_size: "9", text_anchor: "middle", "{lbl_blue}" }
                text { x: "{size - 8}", y: "{half - 4.0}", fill: "#c25244", font_size: "9", text_anchor: "end", "{lbl_red}" }
                text { x: "8", y: "{half - 4.0}", fill: "#438a56", font_size: "9", text_anchor: "start", "{lbl_green}" }

                // Palette gamut polygon
                if hull_ab.len() >= 3 {
                    polygon {
                        points: "{polygon_points}",
                        fill: "rgba(226, 160, 95, 0.18)",
                        stroke: "#c07a3e",
                        stroke_width: "1.5",
                    }
                }

                // Vector between target and achieved mix
                if let (Some((tx, ty, _, _)), Some((ax, ay, _))) = (&target_pt, &achieved_pt) {
                    line {
                        x1: "{tx}",
                        y1: "{ty}",
                        x2: "{ax}",
                        y2: "{ay}",
                        stroke: "#d9705f",
                        stroke_width: "1.5",
                        stroke_dasharray: "3,2",
                    }
                }

                // Palette tubes
                for (tx, ty, code, hex, _name) in tube_markers {
                    g {
                        circle {
                            cx: "{tx}",
                            cy: "{ty}",
                            r: "6.5",
                            fill: "{hex}",
                            stroke: "#f5f0ea",
                            stroke_width: "1.2",
                        }
                        text {
                            x: "{tx + 8.5}",
                            y: "{ty + 3.0}",
                            fill: "{code_fill}",
                            font_size: "9.5",
                            font_weight: "700",
                            "{code}"
                        }
                    }
                }

                // Achieved mix
                if let Some((ax, ay, ahex)) = achieved_pt {
                    circle {
                        cx: "{ax}",
                        cy: "{ay}",
                        r: "5.5",
                        fill: "{ahex}",
                        stroke: "#48a890",
                        stroke_width: "2",
                    }
                }

                // Target reticle
                if let Some((tx, ty, thex, _)) = &target_pt {
                    g {
                        circle {
                            cx: "{tx}",
                            cy: "{ty}",
                            r: "7.5",
                            fill: "{thex}",
                            stroke: "#ffffff",
                            stroke_width: "2",
                        }
                        circle {
                            cx: "{tx}",
                            cy: "{ty}",
                            r: "10.5",
                            fill: "none",
                            stroke: "#c07a3e",
                            stroke_width: "1.2",
                        }
                    }
                }
            }
            if let Some((_, _, _, inside)) = target_pt {
                if hull_ab.len() >= 3 {
                    if inside {
                        span {
                            class: "pill pill-ok",
                            "{locale.tr(\"Cible dans le gamut de la palette\", \"Target inside palette gamut\")}"
                        }
                    } else {
                        span {
                            class: "pill pill-warn",
                            "{locale.tr(\"Cible hors-gamut (saturation max atteinte)\", \"Target out of gamut (max chroma reached)\")}"
                        }
                    }
                }
            }
        }
    }
}

/// Maximum image file size accepted from the file picker (50 MiB).
const MAX_UI_IMAGE_FILE_BYTES: u64 = 50 * 1024 * 1024;

/// Reads an image file selected by the user after verifying its size does not exceed 50 MiB.
pub fn read_image_file_bounded(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let meta = std::fs::metadata(path)
        .map_err(|e| format!("Impossible de lire les métadonnées du fichier : {e}"))?;
    if meta.len() > MAX_UI_IMAGE_FILE_BYTES {
        return Err(format!(
            "Fichier image trop volumineux ({} Mo > limite 50 Mo).",
            meta.len() / (1024 * 1024)
        ));
    }
    std::fs::read(path).map_err(|e| format!("Impossible de lire le fichier image : {e}"))
}

/// Exports a `MixingGuide` as a standalone A4 HTML sheet and opens it in the default browser.
pub fn export_and_open_guide_html(
    guide: &palettier_domain::guide::MixingGuide,
    photo_data_uri: Option<&str>,
    locale: palettier_domain::i18n::Locale,
) -> Result<std::path::PathBuf, String> {
    let html = guide.to_printable_html(photo_data_uri, locale);
    let safe_name: String = guide
        .subtitle
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .take(6)
        .collect::<Vec<_>>()
        .join("-");
    let filename = if safe_name.is_empty() {
        "palettier-mixing-guide.html".to_string()
    } else {
        format!("palettier-guide-{safe_name}.html")
    };
    let out_dir = dirs::download_dir()
        .or_else(dirs::document_dir)
        .unwrap_or_else(std::env::temp_dir);
    let out_path = out_dir.join(filename);
    std::fs::write(&out_path, html).map_err(|e| format!("Failed to write file: {e}"))?;
    open_path_in_default_app(&out_path);
    Ok(out_path)
}

fn open_path_in_default_app(path: &std::path::Path) {
    if cfg!(test) {
        return;
    }
    #[cfg(target_os = "macos")]
    let res = std::process::Command::new("open").arg(path).spawn();
    #[cfg(target_os = "linux")]
    let res = std::process::Command::new("xdg-open").arg(path).spawn();
    #[cfg(target_os = "windows")]
    let res = std::process::Command::new("cmd")
        .args(["/C", "start", ""])
        .arg(path)
        .spawn();
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let res: std::io::Result<std::process::Child> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "unsupported platform",
    ));
    if let Err(e) = res {
        tracing::warn!("Failed to open exported file {:?}: {e}", path);
    }
}

/// Component rendering the full workbench "Mixing Guide" (inspired by the Norma Blue sheets in `references/`)
/// with a 1-click A4 PDF/HTML print button.
#[component]
pub fn MixingGuideCard(
    guide: palettier_domain::guide::MixingGuide,
    photo_data_uri: Option<String>,
    on_notify: EventHandler<String>,
) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    let guide_for_export = guide.clone();
    let photo_for_export = photo_data_uri.clone();

    let header_label = locale.tr(
        "Fiche d'établi · Format Guide de mélanges",
        "Workbench Sheet · Mixing Guide Format",
    );
    let print_btn_label = locale.tr("Imprimer / Exporter en PDF A4", "Print / Export A4 PDF");
    let palette_heading = locale.tr("Ta palette", "Your Palette");
    let mother_heading = locale.tr(
        "Mélanges intermédiaires à préparer en début de séance",
        "Intermediate Mother Mixes to Prepare Before Painting",
    );
    let washes_heading = locale.tr(
        "Lavis sur sous-couche blanche (à l'huile)",
        "Oil Washes Over White Undercoat",
    );
    let method_label = locale.tr("Méthode : ", "Method: ");
    let tips_heading = locale.tr("Conseils", "Studio Tips");

    rsx! {
        div { class: "guide-sheet",
            div { class: "row-between u-mb-14 u-bb-1-solid-he2d9c8 u-pb-10",
                span { class: "u-fs-11p5 u-fw-700 u-c-h7c2d12 u-tt-uppercase u-ls-0p6",
                    "{header_label}"
                }
                button {
                    class: "btn-primary",
                    class: "u-bg-h7c2d12 u-c-hffffff u-bc-h7c2d12",
                    onclick: move |_| {
                        match export_and_open_guide_html(&guide_for_export, photo_for_export.as_deref(), locale) {
                            Ok(path) => {
                                let msg = match locale {
                                    Locale::Fr => format!("Fiche A4 ouverte et enregistrée : {}", path.display()),
                                    Locale::En => format!("A4 sheet opened and saved: {}", path.display()),
                                };
                                on_notify.call(msg);
                            }
                            Err(e) => on_notify.call(e),
                        }
                    },
                    "{print_btn_label}"
                }
            }

            div { class: "u-d-flex u-gap-22 u-ai-flex-start u-fwrap-wrap",
                if let Some(ref uri) = photo_data_uri {
                    img {
                        src: "{uri}",
                        class: "u-w-165 u-maxh-215 u-of-cover u-r-4 u-b-1-solid-hc8bfb0",
                    }
                }
                div { class: "u-flex-1 u-minw-280",
                    h1 { class: "guide-title", "{guide.title}" }
                    p { class: "guide-sub", "{guide.subtitle}" }
                    p { class: "guide-disclaimer", "{guide.disclaimer}" }

                    if !guide.palette_entries.is_empty() {
                        h2 { "{palette_heading}" }
                        table { class: "guide-tbl",
                            tbody {
                                for entry in &guide.palette_entries {
                                    tr {
                                        td { class: "u-w-44",
                                            span {
                                                class: "guide-swatch",
                                                style: "background-color: {entry.paint.hex_color};",
                                            }
                                        }
                                        td { class: "u-w-42 u-fw-750", "{entry.short_code}" }
                                        td {
                                            "{entry.paint.name}"
                                            if !entry.paint.code.trim().is_empty() {
                                                span { class: "u-c-h78716a u-ml-5", "({entry.paint.code})" }
                                            }
                                        }
                                        td { class: "u-c-h57524a u-fs-11p5 u-w-35pct", "{entry.behavior}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !guide.mother_mixes.is_empty() {
                h2 { "{mother_heading}" }
                table { class: "guide-tbl",
                    tbody {
                        for mm in &guide.mother_mixes {
                            tr {
                                td { class: "u-w-44",
                                    span {
                                        class: "guide-swatch",
                                        style: "background-color: {mm.hex_color};",
                                    }
                                }
                                td { class: "u-w-140 u-fw-750", "{mm.code} — {mm.name}" }
                                td { "{mm.recipe_text}" }
                            }
                        }
                    }
                }
                p { class: "u-fs-11p5 u-c-h4a453e u-m-6-0-12-0",
                    "{guide.mother_mixes_note}"
                }
            }

            if !guide.washes.is_empty() {
                h2 { "{washes_heading}" }
                table { class: "guide-tbl",
                    tbody {
                        for w in &guide.washes {
                            tr {
                                td { class: "u-w-44",
                                    span {
                                        class: "guide-swatch",
                                        style: "background-color: {w.hex_color};",
                                    }
                                }
                                td { class: "u-w-155 u-fw-750", "{w.name}" }
                                td { "{w.recipe_text}" }
                            }
                        }
                    }
                }
                p { class: "u-fs-11p5 u-c-h4a453e u-m-6-0-12-0",
                    strong { "{method_label}" }
                    "{guide.washes_method}"
                }
            }

            for zone in &guide.zones {
                h2 {
                    if zone.material.trim().is_empty() || zone.name.to_lowercase().contains(&zone.material.to_lowercase()) {
                        "{zone.name}"
                    } else {
                        "{zone.name} ({zone.material})"
                    }
                }
                table { class: "guide-tbl",
                    tbody {
                        for row in &zone.rows {
                            tr {
                                td { class: "u-w-44",
                                    span {
                                        class: "guide-swatch",
                                        style: "background-color: {row.swatch_hex};",
                                        title: "{row.swatch_hex} / {row.target_hex}",
                                    }
                                }
                                td { class: "u-w-140 u-fw-750", "{row.role}" }
                                td { "{row.painter_recipe}" }
                            }
                        }
                    }
                }
            }

            if !guide.advice.is_empty() {
                h2 { "{tips_heading}" }
                ol { class: "u-m-6-0-0-0 u-pl-20 u-fs-12p5 u-c-h262320",
                    for tip in &guide.advice {
                        li { class: "u-mb-5", "{tip}" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::read_image_file_bounded;

    #[test]
    fn read_image_file_bounded_accepts_small_and_rejects_oversized_files() {
        let dir = std::env::temp_dir().join(format!(
            "palettier-ui-img-test-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();

        let small = dir.join("small.bin");
        std::fs::write(&small, b"hello").unwrap();
        assert_eq!(b"hello", read_image_file_bounded(&small).unwrap().as_slice());

        let oversized = dir.join("oversized.bin");
        let f = std::fs::File::create(&oversized).unwrap();
        f.set_len(51 * 1024 * 1024).unwrap();
        assert!(read_image_file_bounded(&oversized).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
