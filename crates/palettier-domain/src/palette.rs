use crate::color::{Colorant, Rgb, delta_e_2000};
use crate::drying::{DryingContext, DryingTimeService, Workshop};
use crate::paint::{DryingClass, LayerThickness, Medium, Paint};
use chrono::{DateTime, Utc};
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A named selection of tubes prepared for a given subject.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Palette {
    pub id: Option<i64>,
    pub name: String,
    pub purpose: String,
    pub notes: String,
    pub paints: Vec<Paint>,
}

impl Palette {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            purpose: String::new(),
            notes: String::new(),
            paints: Vec::new(),
        }
    }

    pub fn slowest_drying_class(&self) -> DryingClass {
        self.paints
            .iter()
            .fold(DryingClass::Fast, |slowest, p| slowest.slowest(p.drying_class))
    }

    pub fn pigments(&self) -> IndexSet<String> {
        let mut set = IndexSet::new();
        for p in &self.paints {
            for code in &p.pigments {
                set.insert(code.clone());
            }
        }
        set
    }

    pub fn with_paint(&self, paint: Paint) -> Self {
        if self
            .paints
            .iter()
            .any(|p| p.id.is_some() && p.id == paint.id)
        {
            self.clone()
        } else {
            let mut next = self.clone();
            next.paints.push(paint);
            next
        }
    }

    pub fn without_paint(&self, paint: &Paint) -> Self {
        let mut next = self.clone();
        next.paints
            .retain(|p| !(p.id.is_some() && p.id == paint.id));
        next
    }

    /// Generates a comprehensive 2-tube subtractive cross-product matrix.
    pub fn generate_matrix(&self) -> PaletteMatrix {
        let n = self.paints.len();
        let mut rows = Vec::with_capacity(n);
        let titanium_white = Colorant::of_masstone(Rgb::new(0.96, 0.96, 0.95));

        for (i, p1) in self.paints.iter().enumerate() {
            let mut row = Vec::with_capacity(n);
            let c1 = p1.colorant();

            for (j, p2) in self.paints.iter().enumerate() {
                let is_diagonal = i == j;
                let c2 = p2.colorant();

                let pure_rgb = if is_diagonal {
                    p1.color()
                } else {
                    Colorant::mix(&[c1, c2], &[0.5, 0.5]).unwrap_or_else(|_| p1.color())
                };

                let tint_rgb = if is_diagonal {
                    Colorant::mix(&[c1, titanium_white], &[0.2, 0.8]).unwrap_or(pure_rgb)
                } else {
                    Colorant::mix(&[c1, c2, titanium_white], &[0.1, 0.1, 0.8]).unwrap_or(pure_rgb)
                };

                let delta_e_a = delta_e_2000(pure_rgb, p1.color());
                let delta_e_b = delta_e_2000(pure_rgb, p2.color());

                row.push(MatrixMixCell {
                    paint_a_idx: i,
                    paint_b_idx: j,
                    is_diagonal,
                    pure_mix_hex: pure_rgb.to_hex(),
                    tint_mix_hex: tint_rgb.to_hex(),
                    delta_e_a,
                    delta_e_b,
                });
            }
            rows.push(row);
        }

        PaletteMatrix {
            palette_name: self.name.clone(),
            paints: self.paints.clone(),
            rows,
        }
    }
}

/// A single cell in the palette mixing matrix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixMixCell {
    pub paint_a_idx: usize,
    pub paint_b_idx: usize,
    pub is_diagonal: bool,
    pub pure_mix_hex: String,
    pub tint_mix_hex: String,
    pub delta_e_a: f64,
    pub delta_e_b: f64,
}

/// A 2D mixing grid across all tubes of a palette.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaletteMatrix {
    pub palette_name: String,
    pub paints: Vec<Paint>,
    pub rows: Vec<Vec<MatrixMixCell>>,
}

/// A mixture laid out on the palette, with the timestamp when it was mixed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaletteMix {
    pub id: Option<i64>,
    pub name: String,
    pub hex_color: String,
    pub recipe: String,
    pub drying_class: DryingClass,
    pub mixed_at: DateTime<Utc>,
    pub workshop: Workshop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixState {
    pub mix: PaletteMix,
    pub workable: Duration,
    pub unusable: Duration,
}

impl MixState {
    pub fn is_open(&self) -> bool {
        !self.workable.is_zero()
    }

    pub fn is_spent(&self) -> bool {
        self.unusable.is_zero()
    }
}

#[derive(Debug, Clone, Default)]
pub struct PaletteMixService {
    drying_time_service: DryingTimeService,
}

impl PaletteMixService {
    const ON_THE_PALETTE: LayerThickness = LayerThickness::Thick;

    pub fn new(drying_time_service: DryingTimeService) -> Self {
        Self {
            drying_time_service,
        }
    }

    pub fn states(&self, mixes: &[PaletteMix], now: DateTime<Utc>) -> Vec<MixState> {
        mixes.iter().map(|m| self.state(m, now)).collect()
    }

    pub fn state(&self, mix: &PaletteMix, now: DateTime<Utc>) -> MixState {
        let context = DryingContext {
            drying_class: mix.drying_class,
            medium: Medium::None,
            medium_ratio: 0.0,
            thickness: Self::ON_THE_PALETTE,
            workshop: mix.workshop,
        };
        let estimate = self.drying_time_service.estimate(&context);
        let age = (now - mix.mixed_at).to_std().unwrap_or(Duration::ZERO);
        MixState {
            mix: mix.clone(),
            workable: estimate.open_time.saturating_sub(age),
            unusable: estimate.touch_dry.saturating_sub(age),
        }
    }

    pub fn spent(&self, mixes: &[PaletteMix], now: DateTime<Utc>) -> Vec<PaletteMix> {
        self.states(mixes, now)
            .into_iter()
            .filter(|s| s.is_spent())
            .map(|s| s.mix)
            .collect()
    }
}
