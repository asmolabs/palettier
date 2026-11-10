# Palettier

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![Safety](https://img.shields.io/badge/safety-%23!%5Bforbid(unsafe__code)%5D-brightgreen.svg)](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

**Palettier** is an advanced oil painting studio assistant for miniature painters, bust sculptors, scale modellers, and traditional oil artists.

Unlike acrylics, oil painting is fundamentally subtractive, drying is measured in days rather than seconds, and recipes are not mere layer lists — they are complex studio schedules governed by pigment chemistry, film thickness, workshop climate, and the **Fat Over Lean** rule.

---

## Key Features

- **Workshop & Today Planning**: Real-time tracking of miniature pieces, active coats, open blending windows, and recoat readiness.
- **Subtractive Color Mixer**: Kubelka-Munk optical modeling, delta-E 2000 matching against 680 real oil paint tubes across 8 major brands (Winsor & Newton, Schmincke Mussini, Gamblin, Michael Harding, Old Holland, Rembrandt, Sennelier, Williamsburg).
- **Fat-Over-Lean Inspector**: Automatic detection of brittle-over-flexible layering and premature cracking hazards.
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
