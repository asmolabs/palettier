# Palettier

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![Safety](https://img.shields.io/badge/safety-%23!%5Bforbid(unsafe__code)%5D-brightgreen.svg)](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

**Palettier** is a studio assistant for miniature painters, bust sculptors and scale modellers working in **oil, acrylic, or acrylic finished in oil**.

## Oil or acrylic

Every project is painted in one **paint system** — `Oil` or `Acrylic` — chosen when it is created. The system decides what the application offers and which rules it applies:

| | Oil | Acrylic |
|---|---|---|
| Thinners & mediums | odorless thinner, white spirit, turpentine, alkyd, linseed / walnut / stand oil, drier | water, acrylic glaze medium, flow improver, retarder |
| Own techniques | oil wash, wet-on-wet blending, dot fading, streaking, OPR | layering, wet blending (with retarder), drybrush, edge highlight, shade wash, contrast paint |
| Shared techniques | pin wash, filter, glaze, base coat, highlights — with system-specific thinner, ratios and tips | |
| Drying model | oxidation, driven by the pigment, in hours and days | evaporation, driven by thickness, humidity and air flow, in minutes |
| Rules | fat over lean | none between acrylic coats |
| Catalog | 680 tubes, 8 oil ranges | 71 reference pots (Citadel, Vallejo Model / Game Color, Army Painter, AK 3rd Gen), partial selection |

**Mixed work** (acrylic base, oil weathering on top) is supported layer by layer: a technique that belongs to one system keeps it, so an *oil wash* on an acrylic piece is still timed and checked as oil. Oil over dry acrylic is the classic workflow; **acrylic over oil is flagged**, since acrylic does not grip an oily film.

Two rules hold everywhere: **a mix never crosses systems** (the mixer searches each system on its own), and the screens that are not tied to a project (catalog, mixer, drying, recipes, miniature tools) follow the **Oil / Acrylic switch** in the top bar, which opening a project sets to that project's system.

Existing databases are migrated in place: every tube, project and palette mix created before the switch is marked as oil.

---

## Key Features

- **Workshop & Today Planning**: Real-time tracking of miniature pieces, active coats, open blending windows, and recoat readiness.
- **Subtractive Color Mixer**: Kubelka-Munk optical modeling, delta-E 2000 matching against 680 oil tubes across 8 ranges (Winsor & Newton Artists' and Winton, Schmincke Norma Professional and Norma Blue, Gamblin, Williamsburg, Scale75 Floww, Abteilung 502) and 71 miniature acrylics.
- **Fat-Over-Lean Inspector**: Detection of lean-over-fat oil coats, and of acrylic laid over oil.
- **Multimodal AI Assistant**: Zone plan generation from reference photos and AI-powered tube recognition from workshop shelf photos.
- **1-Click Day / Night Mode**: Instant UI theme switching (`Light` / `Dark`) persisted in SQLite.
- **Internationalization (i18n)**: Seamless bilingual support (`FR` / `EN`) with instant language toggle and localized printable workshop guide exports (HTML/PDF).

---

## Workspace Architecture

All crates enforce `#![forbid(unsafe_code)]`:

```
├── crates/
│   ├── palettier-domain/   # Core domain entities, Kubelka-Munk physics, drying models, fat-over-lean rules, i18n
│   ├── palettier-image/    # Pixel decoding, linear light downsampling, k-means color extraction, grisaille/notan
│   ├── palettier-data/     # SeaORM & SQLite persistence, catalog seeder, ZIP backup import/export
│   ├── palettier-ai/       # Multimodal vision (Gemini/OpenAI/Claude/Ollama), plan generation & tube recognition
│   ├── palettier-ui/       # Native desktop application built with Dioxus 0.6 Desktop
│   └── palettier-cli/      # Command-line interface for batch tasks, catalog queries, and CLI workflows
├── Cargo.toml              # Cargo workspace configuration
├── Dioxus.toml             # Dioxus desktop bundle configuration
└── references/             # Test photos and studio calibration images
```

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (2024 edition / stable toolchain)

### Running the Desktop App

```bash
cargo run -p palettier-ui
```

### Running the CLI

```bash
cargo run -p palettier-cli -- help
```

### Running Tests

```bash
cargo test --workspace
```

---

## License

Dual-licensed under either of:
- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)
