//! Global CSS stylesheet (supporting Night/Dark and Day/Light studio themes) and reusable UI components.

use crate::state::{AppState, ThemeMode};
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;

pub const APP_CSS: &str = r#"
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body, html {
    font-family: "SF Pro Text", "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    font-size: 13.5px;
    line-height: 1.45;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    user-select: none;
}

/* Night Mode (Dark Studio) */
.app-shell,
.app-shell.theme-dark {
    --bg-app: #0e0d0c;
    --bg-sidebar: #131211;
    --bg-card: #191715;
    --bg-subtle: #141210;
    --bg-input: #201d1b;
    --bg-hover: #262220;
    --bg-active: #2a2320;
    --bg-toast: #262019;
    --text-main: #eee9e3;
    --text-muted: #928a81;
    --text-dim: #6d655d;
    --border-main: #2b2724;
    --border-subtle: #24201c;
    --border-hover: #3b3531;
    --accent: #e2a05f;
    --accent-hover: #edae6d;
    --accent-contrast: #1b1410;
    --accent-bg: rgba(226, 160, 95, 0.16);
    --accent-border: rgba(226, 160, 95, 0.35);
    --pill-bg: rgba(255, 255, 255, 0.06);
    --pill-text: #d6cec5;
    --pill-ok-bg: rgba(111, 181, 165, 0.16);
    --pill-ok-text: #7fc6b5;
    --pill-ok-border: rgba(111, 181, 165, 0.35);
    --pill-warn-bg: rgba(229, 169, 85, 0.16);
    --pill-warn-text: #e5a955;
    --pill-warn-border: rgba(229, 169, 85, 0.35);
    --pill-danger-bg: rgba(217, 112, 95, 0.20);
    --pill-danger-text: #f0a496;
    --pill-danger-border: rgba(217, 112, 95, 0.40);
    --swatch-border: rgba(255, 255, 255, 0.18);
    --row-divider: rgba(255, 255, 255, 0.05);
}

/* Day Mode (Warm Daylight Studio) */
.app-shell.theme-light {
    --bg-app: #f4efe6;
    --bg-sidebar: #eae3d5;
    --bg-card: #ffffff;
    --bg-subtle: #f9f6f0;
    --bg-input: #ffffff;
    --bg-hover: #e2d9c8;
    --bg-active: #dfd3be;
    --bg-toast: #fff8ed;
    --text-main: #1f1c18;
    --text-muted: #5c554c;
    --text-dim: #7a7166;
    --border-main: #d5ccbc;
    --border-subtle: #e4ddd0;
    --border-hover: #b8ae9c;
    --accent: #8c3516;
    --accent-hover: #74290e;
    --accent-contrast: #ffffff;
    --accent-bg: rgba(140, 53, 22, 0.11);
    --accent-border: rgba(140, 53, 22, 0.30);
    --pill-bg: rgba(31, 28, 24, 0.06);
    --pill-text: #3a342d;
    --pill-ok-bg: rgba(22, 121, 95, 0.12);
    --pill-ok-text: #136851;
    --pill-ok-border: rgba(22, 121, 95, 0.32);
    --pill-warn-bg: rgba(180, 105, 14, 0.13);
    --pill-warn-text: #945208;
    --pill-warn-border: rgba(180, 105, 14, 0.34);
    --pill-danger-bg: rgba(185, 48, 32, 0.12);
    --pill-danger-text: #a32618;
    --pill-danger-border: rgba(185, 48, 32, 0.32);
    --swatch-border: rgba(0, 0, 0, 0.22);
    --row-divider: rgba(0, 0, 0, 0.06);
}

.app-shell {
    display: flex;
    height: 100vh;
    width: 100vw;
    background-color: var(--bg-app);
    color: var(--text-main);
    transition: background-color 0.18s ease, color 0.18s ease;
}

/* Sidebar */
.sidebar {
    width: 256px;
    min-width: 256px;
    background-color: var(--bg-sidebar);
    border-right: 1px solid var(--border-main);
    display: flex;
    flex-direction: column;
    padding: 18px 14px 16px 14px;
    transition: background-color 0.18s ease, border-color 0.18s ease;
}

.brand-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 10px 14px 10px;
    border-bottom: 1px solid var(--border-main);
    margin-bottom: 8px;
}

.app-mark {
    width: 36px;
    height: 36px;
    min-width: 36px;
    border-radius: 10px;
    background: linear-gradient(135deg, #e2a05f 0%, #c07a3e 48%, #6d321c 100%);
    border: 1px solid rgba(255, 255, 255, 0.25);
    box-shadow: 0 3px 10px rgba(0, 0, 0, 0.28);
}

.brand-title {
    font-size: 17px;
    font-weight: 700;
    color: var(--text-main);
    letter-spacing: -0.3px;
}

.brand-subtitle {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 1px;
}

.quick-toggles {
    display: flex;
    gap: 6px;
    padding: 4px 6px 10px 6px;
    border-bottom: 1px solid var(--border-main);
    margin-bottom: 6px;
}

.toggle-chip {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    background-color: var(--bg-input);
    color: var(--text-main);
    border: 1px solid var(--border-main);
    border-radius: 7px;
    padding: 5.5px 8px;
    font-size: 11.5px;
    font-weight: 650;
    cursor: pointer;
    transition: all 0.12s ease;
}

.toggle-chip:hover {
    background-color: var(--bg-hover);
    border-color: var(--accent);
}

.toggle-chip.active {
    background-color: var(--accent);
    color: var(--accent-contrast);
    border-color: var(--accent);
}

/* Painter HUD (Distraction-Free Workbench Mode) */
.painter-hud {
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background-color: var(--bg-app);
    color: var(--text-main);
    overflow: hidden;
}

.painter-hud-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
    background-color: var(--bg-sidebar);
    border-bottom: 2px solid var(--accent);
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.35);
    z-index: 10;
}

.painter-hud-body {
    display: grid;
    grid-template-columns: 1.2fr 0.8fr;
    gap: 16px;
    padding: 16px;
    height: calc(100vh - 58px);
    overflow: hidden;
}

.painter-hud-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    background-color: var(--bg-card);
    border: 1px solid var(--border-main);
    border-radius: 10px;
    padding: 16px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
}

.hud-swatch-large {
    width: 46px;
    height: 46px;
    min-width: 46px;
    border-radius: 8px;
    border: 2px solid var(--swatch-border);
    box-shadow: 0 3px 8px rgba(0, 0, 0, 0.3);
}

.hud-mix-card {
    background: var(--bg-subtle);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    padding: 12px;
    margin-bottom: 12px;
    transition: border-color 0.15s ease;
}

.hud-mix-card:hover {
    border-color: var(--accent);
}

.nav-group-label {
    font-size: 10px;
    font-weight: 700;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.9px;
    padding: 11px 10px 5px 10px;
}

.nav-btn {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    background: transparent;
    color: var(--text-muted);
    border: none;
    border-radius: 9px;
    padding: 8.5px 11px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    text-align: left;
    transition: all 0.12s ease;
    margin-bottom: 2px;
}

.nav-btn-left {
    display: flex;
    align-items: center;
    gap: 10px;
}

.nav-dot {
    width: 7px;
    height: 7px;
    border-radius: 999px;
    background-color: var(--border-main);
    transition: all 0.12s ease;
}

.nav-btn:hover {
    background-color: var(--bg-hover);
    color: var(--text-main);
}

.nav-btn:hover .nav-dot {
    background-color: var(--text-muted);
}

.nav-btn.active {
    background-color: var(--bg-active);
    color: var(--accent);
}

.nav-btn.active .nav-dot {
    background-color: var(--accent);
    box-shadow: 0 0 8px var(--accent-border);
}

.nav-shortcut {
    font-size: 10.5px;
    color: var(--text-dim);
    background: var(--bg-subtle);
    padding: 1px 6px;
    border-radius: 5px;
    border: 1px solid var(--border-main);
}

.nav-btn.active .nav-shortcut {
    color: var(--accent);
    border-color: var(--accent-border);
}

.sidebar-footer {
    margin-top: auto;
    padding-top: 12px;
    border-top: 1px solid var(--border-main);
    font-size: 11px;
    color: var(--text-dim);
}

/* Main Content */
.main-pane {
    flex: 1;
    overflow-y: auto;
    padding: 24px 34px;
    background-color: var(--bg-app);
    transition: background-color 0.18s ease;
}

.section-header {
    margin-bottom: 20px;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
}

.page-title {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-main);
    letter-spacing: -0.4px;
}

.page-subtitle {
    font-size: 12.5px;
    color: var(--text-muted);
    margin-top: 3px;
    max-width: 760px;
}

/* Cards & Grid */
.card {
    background-color: var(--bg-card);
    border: 1px solid var(--border-main);
    border-radius: 13px;
    padding: 18px 20px;
    margin-bottom: 16px;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.04);
}

.card-subtle {
    background-color: var(--bg-subtle);
    border: 1px solid var(--border-subtle);
    border-radius: 10px;
    padding: 12px 15px;
}

.card-title {
    font-size: 14.5px;
    font-weight: 700;
    color: var(--text-main);
    margin-bottom: 10px;
}

.grid-2 {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 16px;
}

.grid-3 {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 16px;
}

.row {
    display: flex;
    align-items: center;
    gap: 9px;
    flex-wrap: wrap;
}

.row-between {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
}

/* Inputs & Buttons */
input[type="text"],
input[type="password"],
input[type="number"],
textarea,
select {
    background-color: var(--bg-input);
    color: var(--text-main);
    border: 1px solid var(--border-main);
    border-radius: 8px;
    padding: 7px 11px;
    font-size: 13px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.12s ease;
}

input:focus, textarea:focus, select:focus {
    border-color: var(--accent);
}

input[type="color"] {
    -webkit-appearance: none;
    appearance: none;
    background: transparent;
    border: 1px solid var(--swatch-border);
    border-radius: 7px;
    width: 30px;
    height: 30px;
    padding: 0;
    cursor: pointer;
}

input[type="color"]::-webkit-color-swatch-wrapper {
    padding: 2px;
}

input[type="color"]::-webkit-color-swatch {
    border: none;
    border-radius: 5px;
}

.field-label {
    display: block;
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-muted);
    margin-bottom: 5px;
}

.btn {
    background-color: var(--bg-input);
    color: var(--text-main);
    border: 1px solid var(--border-main);
    border-radius: 8px;
    padding: 7px 14px;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s ease;
}

.btn:hover:not(:disabled) {
    background-color: var(--bg-hover);
    border-color: var(--border-hover);
}

.btn:disabled {
    opacity: 0.5;
    cursor: default;
}

.btn-primary {
    background-color: var(--accent);
    color: var(--accent-contrast);
    border: 1px solid var(--accent);
    border-radius: 8px;
    padding: 7px 15px;
    font-size: 12.5px;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.12s ease;
}

.btn-primary:hover:not(:disabled) {
    background-color: var(--accent-hover);
}

.btn-danger {
    background-color: var(--pill-danger-bg);
    color: var(--pill-danger-text);
    border: 1px solid var(--pill-danger-border);
    border-radius: 8px;
    padding: 5px 11px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
}

.btn-danger:hover {
    opacity: 0.85;
}

/* Pills & Badges */
.pill {
    display: inline-flex;
    align-items: center;
    padding: 2px 9px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 600;
    background-color: var(--pill-bg);
    color: var(--pill-text);
    border: 1px solid var(--border-main);
}

.pill-accent {
    background-color: var(--accent-bg);
    color: var(--accent);
    border-color: var(--accent-border);
}

.pill-ok {
    background-color: var(--pill-ok-bg);
    color: var(--pill-ok-text);
    border-color: var(--pill-ok-border);
}

.pill-warn {
    background-color: var(--pill-warn-bg);
    color: var(--pill-warn-text);
    border-color: var(--pill-warn-border);
}

.pill-danger {
    background-color: var(--pill-danger-bg);
    color: var(--pill-danger-text);
    border-color: var(--pill-danger-border);
}

/* Swatches & Value Ladder */
.swatch {
    display: inline-block;
    border-radius: 7px;
    border: 1px solid var(--swatch-border);
    flex-shrink: 0;
}

.split-swatch {
    display: inline-flex;
    border-radius: 7px;
    overflow: hidden;
    border: 1px solid var(--swatch-border);
    flex-shrink: 0;
}

.ladder-strip {
    display: flex;
    width: 100%;
    height: 36px;
    border-radius: 9px;
    overflow: hidden;
    border: 1px solid var(--swatch-border);
    margin: 8px 0 12px 0;
}

.ladder-step {
    flex: 1;
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    padding: 4px 8px;
    font-size: 10.5px;
    font-weight: 700;
    cursor: pointer;
    transition: filter 0.12s ease;
}

.ladder-step:hover {
    filter: brightness(1.08);
}

/* Share bars (Volume / Pigment weight) */
.share-bar-bg {
    width: 100%;
    height: 7px;
    background-color: var(--border-subtle);
    border-radius: 999px;
    overflow: hidden;
    margin-top: 3px;
}

.share-bar-fill {
    height: 100%;
    background: linear-gradient(90deg, #c07a3e, #e2a05f);
    border-radius: 999px;
}

/* Eyedropper interactive canvas */
.eyedropper-viewport {
    position: relative;
    background-color: var(--bg-subtle);
    border: 1px solid var(--border-main);
    border-radius: 10px;
    overflow: auto;
    max-height: 520px;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 8px;
}

.eyedropper-img {
    cursor: crosshair;
    max-width: 100%;
    height: auto;
    border-radius: 6px;
    display: block;
}

/* Photo thumbnail strip */
.photo-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 12px;
}

.photo-thumb {
    width: 100%;
    height: 125px;
    object-fit: cover;
    border-radius: 7px;
    border: 1px solid var(--border-main);
    margin-bottom: 6px;
}

.toast-banner {
    background-color: var(--bg-toast);
    border: 1px solid var(--accent);
    color: var(--text-main);
    padding: 10px 16px;
    border-radius: 9px;
    margin-bottom: 18px;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.muted {
    color: var(--text-muted);
}

.mono {
    font-family: "SF Mono", "JetBrains Mono", Menlo, monospace;
    font-size: 12px;
}

table.data-table {
    width: 100%;
    border-collapse: collapse;
}

table.data-table th {
    text-align: left;
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    color: var(--text-dim);
    padding: 8px 10px;
    border-bottom: 1px solid var(--border-main);
}

table.data-table td {
    padding: 8.5px 10px;
    border-bottom: 1px solid var(--row-divider);
    vertical-align: middle;
}

table.data-table tr:hover td {
    background-color: var(--bg-hover);
}

/* Mixing Guide sheet (Norma Blue workbench style) */
.guide-sheet {
    background-color: #f9f6f0;
    color: #262320;
    border-radius: 12px;
    border: 1px solid #dcd4c6;
    padding: 24px 28px;
    margin-top: 14px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
}

.guide-sheet h2 {
    color: #7c2d12;
    font-size: 16px;
    font-weight: 750;
    margin: 18px 0 8px 0;
}

.guide-sheet .guide-title {
    color: #1f1c18;
    font-size: 24px;
    font-weight: 800;
    margin: 0 0 4px 0;
}

.guide-sheet .guide-sub {
    color: #57524a;
    font-size: 13.5px;
    margin: 0 0 8px 0;
}

.guide-sheet .guide-disclaimer {
    color: #6e675f;
    font-size: 11.5px;
    margin: 0 0 14px 0;
}

.guide-sheet table.guide-tbl {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 8px;
}

.guide-sheet table.guide-tbl tr {
    border-bottom: 1px solid #e6dfd3;
}

.guide-sheet table.guide-tbl td {
    padding: 6px 8px 6px 0;
    vertical-align: middle;
    font-size: 12.5px;
    color: #262320;
}

.guide-swatch {
    width: 36px;
    height: 22px;
    border-radius: 3px;
    border: 1px solid #b5ada0;
    display: inline-block;
}
"#;

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
        let style = format!("width: {width}px; height: {height}px; background-color: {masstone_hex};");
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
                            span { class: "mono", style: "font-size: 9.5px; opacity: 0.85;", "{hex}" }
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
                style: "width: 98px;",
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
            format!("ΔE {:.1} · {}", delta_e, locale.tr("imperceptible", "imperceptible")),
        )
    } else if delta_e < 2.0 {
        (
            "pill pill-ok",
            format!("ΔE {:.1} · {}", delta_e, locale.tr("identique à l'œil", "indistinguishable")),
        )
    } else if delta_e < 4.5 {
        (
            "pill pill-accent",
            format!("ΔE {:.1} · {}", delta_e, locale.tr("très proche", "very close")),
        )
    } else if delta_e < 8.0 {
        (
            "pill pill-warn",
            format!("ΔE {:.1} · {}", delta_e, locale.tr("écart visible", "visible gap")),
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
    use palettier_domain::guide::MixingGuideBuilder;

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

    let entries = MixingGuideBuilder::build_palette_entries(&paints);
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
    };

    let lbl_yellow = locale.tr("+b* Jaune", "+b* Yellow");
    let lbl_blue = locale.tr("-b* Bleu", "-b* Blue");
    let lbl_red = locale.tr("+a* Rouge", "+a* Red");
    let lbl_green = locale.tr("-a* Vert", "-a* Green");

    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 6px;",
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

/// Exports a `MixingGuide` as a standalone A4 HTML sheet and opens it in the default browser.
pub fn export_and_open_guide_html(
    guide: &palettier_domain::guide::MixingGuide,
    photo_data_uri: Option<&str>,
    locale: palettier_domain::i18n::Locale,
) -> Result<std::path::PathBuf, String> {
    let html = guide.to_printable_html_i18n(photo_data_uri, locale);
    let safe_name: String = guide
        .subtitle
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
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
    let _ = std::process::Command::new("open").arg(&out_path).spawn();
    Ok(out_path)
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
    let print_btn_label = locale.tr(
        "Imprimer / Exporter en PDF A4",
        "Print / Export A4 PDF",
    );
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
            div { class: "row-between", style: "margin-bottom: 14px; border-bottom: 1px solid #e2d9c8; padding-bottom: 10px;",
                span { style: "font-size: 11.5px; font-weight: 700; color: #7c2d12; text-transform: uppercase; letter-spacing: 0.6px;",
                    "{header_label}"
                }
                button {
                    class: "btn-primary",
                    style: "background: #7c2d12; color: #ffffff; border-color: #7c2d12;",
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

            div { style: "display: flex; gap: 22px; align-items: flex-start; flex-wrap: wrap;",
                if let Some(ref uri) = photo_data_uri {
                    img {
                        src: "{uri}",
                        style: "width: 165px; max-height: 215px; object-fit: cover; border-radius: 4px; border: 1px solid #c8bfb0;",
                    }
                }
                div { style: "flex: 1; min-width: 280px;",
                    h1 { class: "guide-title", "{guide.title}" }
                    p { class: "guide-sub", "{guide.subtitle}" }
                    p { class: "guide-disclaimer", "{guide.disclaimer}" }

                    if !guide.palette_entries.is_empty() {
                        h2 { "{palette_heading}" }
                        table { class: "guide-tbl",
                            tbody {
                                for entry in &guide.palette_entries {
                                    tr {
                                        td { style: "width: 44px;",
                                            span {
                                                class: "guide-swatch",
                                                style: "background-color: {entry.paint.hex_color};",
                                            }
                                        }
                                        td { style: "width: 42px; font-weight: 750;", "{entry.short_code}" }
                                        td {
                                            "{entry.paint.name}"
                                            if !entry.paint.code.trim().is_empty() {
                                                span { style: "color: #78716a; margin-left: 5px;", "({entry.paint.code})" }
                                            }
                                        }
                                        td { style: "color: #57524a; font-size: 11.5px; width: 35%;", "{entry.behavior}" }
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
                                td { style: "width: 44px;",
                                    span {
                                        class: "guide-swatch",
                                        style: "background-color: {mm.hex_color};",
                                    }
                                }
                                td { style: "width: 140px; font-weight: 750;", "{mm.code} — {mm.name}" }
                                td { "{mm.recipe_text}" }
                            }
                        }
                    }
                }
                p { style: "font-size: 11.5px; color: #4a453e; margin: 6px 0 12px 0;",
                    "{guide.mother_mixes_note}"
                }
            }

            if !guide.washes.is_empty() {
                h2 { "{washes_heading}" }
                table { class: "guide-tbl",
                    tbody {
                        for w in &guide.washes {
                            tr {
                                td { style: "width: 44px;",
                                    span {
                                        class: "guide-swatch",
                                        style: "background-color: {w.hex_color};",
                                    }
                                }
                                td { style: "width: 155px; font-weight: 750;", "{w.name}" }
                                td { "{w.recipe_text}" }
                            }
                        }
                    }
                }
                p { style: "font-size: 11.5px; color: #4a453e; margin: 6px 0 12px 0;",
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
                                td { style: "width: 44px;",
                                    span {
                                        class: "guide-swatch",
                                        style: "background-color: {row.swatch_hex};",
                                        title: "{row.swatch_hex} / {row.target_hex}",
                                    }
                                }
                                td { style: "width: 140px; font-weight: 750;", "{row.role}" }
                                td { "{row.painter_recipe}" }
                            }
                        }
                    }
                }
            }

            if !guide.advice.is_empty() {
                h2 { "{tips_heading}" }
                ol { style: "margin: 6px 0 0 0; padding-left: 20px; font-size: 12.5px; color: #262320;",
                    for tip in &guide.advice {
                        li { style: "margin-bottom: 5px;", "{tip}" }
                    }
                }
            }
        }
    }
}
