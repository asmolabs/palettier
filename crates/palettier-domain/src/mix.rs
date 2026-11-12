use crate::color::{Colorant, Lab, Rgb, delta_e_2000, delta_e_2000_lab};
use crate::error::{DomainError, DomainResult};
use crate::i18n::Locale;
use crate::paint::{DryingClass, Opacity, Paint, PaintSystem};
use indexmap::{IndexMap, IndexSet};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::f64::consts::PI;
use std::sync::LazyLock;

/// An oil paint and its proportion in a mix, expressed in parts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaintPart {
    pub paint: Paint,
    pub parts: f64,
}

impl PaintPart {
    pub fn new(paint: Paint, parts: f64) -> DomainResult<Self> {
        if parts <= 0.0 {
            return Err(DomainError::InvalidArgument(
                "La dose doit être strictement positive".to_string(),
            ));
        }
        Ok(Self { paint, parts })
    }

    pub fn of(paint: Paint, parts: f64) -> Self {
        assert!(parts > 0.0, "La dose doit être strictement positive");
        Self { paint, parts }
    }
}

/// Contribution of a single paint to a mix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixComponent {
    pub paint: Paint,
    pub parts: f64,
    pub volume_share: f64,
    pub pigment_share: f64,
}

/// Complete result of a paint mix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixResult {
    pub color: Rgb,
    pub components: Vec<MixComponent>,
    pub drying_class: DryingClass,
    pub pigments: IndexSet<String>,
    pub warnings: Vec<String>,
}

impl MixResult {
    pub fn hex(&self) -> String {
        self.color.to_hex()
    }
}

/// A suggested paint mix for approximating a target color.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixSuggestion {
    pub parts: Vec<PaintPart>,
    pub color: Rgb,
    pub delta_e: f64,
}

impl MixSuggestion {
    pub fn drying_class(&self) -> DryingClass {
        self.parts.iter().fold(DryingClass::Fast, |slowest, part| {
            slowest.slowest(part.paint.drying_class)
        })
    }

    pub fn describe(&self) -> String {
        self.describe_i18n(Locale::Fr)
    }

    pub fn describe_i18n(&self, locale: Locale) -> String {
        self.parts
            .iter()
            .map(|part| {
                let amount = if (part.parts - part.parts.floor()).abs() < 1e-9 {
                    format!("{}", part.parts as i64)
                } else {
                    format!("{}", part.parts)
                };
                let plural = if part.parts > 1.0 { "s" } else { "" };
                match locale {
                    Locale::Fr => format!("{amount} part{plural} de {}", part.paint.display_name()),
                    Locale::En => format!("{amount} part{plural} {}", part.paint.display_name()),
                }
            })
            .collect::<Vec<_>>()
            .join("  +  ")
    }

    pub fn verdict(&self) -> &'static str {
        self.verdict_i18n(Locale::Fr)
    }

    pub fn verdict_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self.delta_e.floor() as i32 {
                0 | 1 => "Identique à l'œil",
                2 | 3 => "Très proche",
                4..=7 => "Proche, écart visible côte à côte",
                8..=14 => "Même famille de teinte",
                _ => "Couleur différente",
            },
            Locale::En => match self.delta_e.floor() as i32 {
                0 | 1 => "Indistinguishable",
                2 | 3 => "Very close",
                4..=7 => "Close, visible side by side",
                8..=14 => "Same hue family",
                _ => "Different color",
            },
        }
    }
}

/// A catalog paint and its perceptual distance to a target color.
#[derive(Debug, Clone, PartialEq)]
pub struct PaintMatch {
    pub paint: Paint,
    pub delta_e: f64,
}

impl PaintMatch {
    pub fn verdict(&self) -> &'static str {
        self.verdict_i18n(Locale::Fr)
    }

    pub fn verdict_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self.delta_e.floor() as i32 {
                0 | 1 => "Identique a l'oeil",
                2 | 3 => "Tres proche",
                4..=7 => "Proche, ecart visible cote a cote",
                8..=14 => "Meme famille de teinte",
                _ => "Couleur differente",
            },
            Locale::En => match self.delta_e.floor() as i32 {
                0 | 1 => "Indistinguishable",
                2 | 3 => "Very close",
                4..=7 => "Close, visible side by side",
                8..=14 => "Same hue family",
                _ => "Different color",
            },
        }
    }
}

/// Simulates mixing multiple oil paints and suggests recipes to match a target hue.
#[derive(Debug, Clone, Default)]
pub struct ColorMixService;

impl ColorMixService {
    const MUDDY_MIX_THRESHOLD: usize = 4;
    pub const MAX_PAINTS: usize = MixSearch::MAX_PAINTS;
    const SHORT_PALETTE: usize = 8;
    const LARGE_SELECTION: usize = 24;

    pub fn new() -> Self {
        Self
    }

    pub fn recommended_max_paints(candidate_count: usize) -> usize {
        if candidate_count <= Self::SHORT_PALETTE {
            Self::MAX_PAINTS
        } else if candidate_count <= Self::LARGE_SELECTION {
            4
        } else {
            3
        }
    }

    pub fn mix(&self, parts: &[PaintPart]) -> DomainResult<MixResult> {
        if parts.is_empty() {
            return Err(DomainError::InvalidArgument(
                "Un mélange demande au moins une peinture".to_string(),
            ));
        }
        if parts
            .iter()
            .any(|p| p.paint.system != parts[0].paint.system)
        {
            return Err(DomainError::InvalidArgument(
                "On ne mélange pas huile et acrylique : l'une se dilue au solvant, l'autre à l'eau."
                    .to_string(),
            ));
        }

        let total_parts: f64 = parts.iter().map(|p| p.parts).sum();
        let total_pigment: f64 = parts
            .iter()
            .map(|p| p.parts * p.paint.tinting_strength)
            .sum();

        let colorants: Vec<Colorant> = parts.iter().map(|p| p.paint.colorant()).collect();
        let weights: Vec<f64> = parts
            .iter()
            .map(|p| p.parts * p.paint.tinting_strength)
            .collect();
        let mixed = Colorant::mix(&colorants, &weights)?;

        let components: Vec<MixComponent> = parts
            .iter()
            .map(|p| MixComponent {
                paint: p.paint.clone(),
                parts: p.parts,
                volume_share: p.parts / total_parts,
                pigment_share: (p.parts * p.paint.tinting_strength) / total_pigment,
            })
            .collect();

        let drying = parts.iter().fold(DryingClass::Fast, |slowest, p| {
            slowest.slowest(p.paint.drying_class)
        });

        let mut pigments = IndexSet::new();
        for p in parts {
            for code in &p.paint.pigments {
                pigments.insert(code.clone());
            }
        }

        let warnings = Self::warnings_for(&components, drying);
        Ok(MixResult {
            color: mixed,
            components,
            drying_class: drying,
            pigments,
            warnings,
        })
    }

    fn warnings_for(components: &[MixComponent], drying: DryingClass) -> Vec<String> {
        let mut out = Vec::new();
        if components.len() >= Self::MUDDY_MIX_THRESHOLD {
            out.push(format!(
                "{} huiles dans le melange : le resultat va tendre vers le gris. Deux ou trois suffisent presque toujours.",
                components.len()
            ));
        }

        for comp in components
            .iter()
            .filter(|c| c.pigment_share > 0.6 && c.volume_share < 0.25)
        {
            out.push(format!(
                "{} occupe {} % du volume mais {} % de la couleur finale : dosez-la a la pointe du pinceau.",
                comp.paint.name,
                (comp.volume_share * 100.0).round() as i64,
                (comp.pigment_share * 100.0).round() as i64
            ));
        }

        let mixes_opacities = components
            .iter()
            .any(|c| c.paint.opacity == Opacity::Transparent)
            && components
                .iter()
                .any(|c| c.paint.opacity == Opacity::Opaque);
        if mixes_opacities {
            out.push(
                "Melange d'une transparente et d'une opaque : le glacis perdra sa profondeur."
                    .to_string(),
            );
        }

        if drying >= DryingClass::Slow {
            out.push(format!(
                "Sechage {} impose par le pigment le plus lent : prevoyez la couche suivante en consequence.",
                drying.label().to_lowercase()
            ));
        }

        let mut pigments = IndexSet::new();
        for c in components {
            for code in &c.paint.pigments {
                pigments.insert(code.clone());
            }
        }
        if pigments.len() > 4 {
            out.push(format!(
                "{} pigments differents en presence : au-dela de quatre, la teinte devient difficile a reproduire.",
                pigments.len()
            ));
        }
        out
    }

    pub fn suggest_mixes(
        &self,
        target: Rgb,
        candidates: &[Paint],
        max_results: usize,
        max_paints: Option<usize>,
    ) -> Vec<MixSuggestion> {
        if candidates.is_empty() || max_results == 0 {
            return Vec::new();
        }
        // A mix never crosses systems: oil and acrylic do not blend in the pot. When the
        // candidates hold both, each system is searched on its own and the best kept.
        let first = candidates[0].system;
        if candidates.iter().any(|p| p.system != first) {
            let mut merged: Vec<MixSuggestion> = PaintSystem::ALL
                .into_iter()
                .flat_map(|system| {
                    let subset: Vec<Paint> = candidates
                        .iter()
                        .filter(|p| p.system == system)
                        .cloned()
                        .collect();
                    self.suggest_mixes(target, &subset, max_results, max_paints)
                })
                .collect();
            merged.sort_by(|a, b| a.delta_e.total_cmp(&b.delta_e));
            merged.truncate(max_results);
            return merged;
        }
        let limit_paints = max_paints
            .unwrap_or_else(|| Self::recommended_max_paints(candidates.len()))
            .clamp(1, Self::MAX_PAINTS);
        MixSearch::new(target, candidates).search(limit_paints, max_results)
    }
}

static PAIR_RATIOS: LazyLock<Vec<Vec<i32>>> = LazyLock::new(practical_pair_ratios);
static RATIO_GRIDS: LazyLock<IndexMap<usize, Vec<Vec<i32>>>> = LazyLock::new(ratio_grids);

fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn practical_pair_ratios() -> Vec<Vec<i32>> {
    let mut out = Vec::new();
    for a in 1..=8 {
        for b in 1..=8 {
            if gcd(a, b) == 1 {
                out.push(vec![a, b]);
            }
        }
    }
    for tiny in [10, 12, 16, 20, 30] {
        out.push(vec![1, tiny]);
        out.push(vec![tiny, 1]);
    }
    out
}

fn ratio_grids() -> IndexMap<usize, Vec<Vec<i32>>> {
    let mut map = IndexMap::new();
    for count in 3..=MixSearch::MAX_PAINTS {
        let max = match count {
            3 => 5,
            4 => 4,
            _ => 3,
        };
        let mut grid = Vec::new();
        let mut current = vec![0; count];
        fill_ratios(&mut current, 0, max, &mut grid);
        map.insert(count, grid);
    }
    map
}

fn fill_ratios(current: &mut [i32], index: usize, max: i32, out: &mut Vec<Vec<i32>>) {
    if index == current.len() {
        let mut divisor = current[0];
        for &part in current.iter() {
            divisor = gcd(divisor, part);
        }
        if divisor == 1 {
            out.push(current.to_vec());
        }
        return;
    }
    for part in 1..=max {
        current[index] = part;
        fill_ratios(current, index + 1, max, out);
    }
}

#[derive(Clone, Copy)]
struct Candidate<'a> {
    paint: &'a Paint,
    absorption: [f64; 3],
    scattering: [f64; 3],
    tinting: f64,
    lab: Lab,
}

impl<'a> Candidate<'a> {
    fn chroma(&self) -> f64 {
        self.lab.a.hypot(self.lab.b)
    }

    fn hue(&self) -> f64 {
        let degrees = self.lab.b.atan2(self.lab.a) * 180.0 / PI;
        if degrees < 0.0 {
            degrees + 360.0
        } else {
            degrees
        }
    }
}

#[derive(Clone)]
struct Evaluation<'a> {
    paints: Vec<Candidate<'a>>,
    parts: Vec<i32>,
    color: Rgb,
    delta_e: f64,
}

struct MixSearch<'a> {
    target: Lab,
    candidates: Vec<Candidate<'a>>,
}

impl<'a> MixSearch<'a> {
    const PAIRS_KEPT: usize = 60;
    const COARSE_WEIGHTS: [f64; 5] = [0.15, 0.35, 0.5, 0.65, 0.85];
    pub const MAX_PAINTS: usize = 5;
    const STRUCTURAL_PAINTS: usize = 4;
    const HUE_SECTORS: usize = 6;
    const PERCEPTUAL_TIE: f64 = 0.5;
    const DUPLICATE_RESULT: f64 = 0.15;

    fn new(target: Rgb, paints: &'a [Paint]) -> Self {
        let candidates = paints
            .iter()
            .map(|paint| {
                let colorant = paint.colorant();
                Candidate {
                    paint,
                    absorption: colorant.absorption(),
                    scattering: colorant.scattering(),
                    tinting: paint.tinting_strength,
                    lab: paint.color().to_lab(),
                }
            })
            .collect();
        Self {
            target: target.to_lab(),
            candidates,
        }
    }

    fn search(&self, max_paints: usize, max_results: usize) -> Vec<MixSuggestion> {
        let mut found = self.singles();
        if max_paints >= 2 {
            found.extend(self.pairs());
        }
        for count in 3..=max_paints.min(Self::MAX_PAINTS) {
            found.extend(self.combinations(count));
        }
        self.best(found, max_results)
    }

    fn singles(&self) -> Vec<Evaluation<'a>> {
        self.candidates
            .iter()
            .map(|&c| self.evaluate(&[c], &[1]))
            .collect()
    }

    fn pairs(&self) -> Vec<Evaluation<'a>> {
        let size = self.candidates.len();
        let mut coarse: Vec<(usize, usize, f64)> = (0..size)
            .into_par_iter()
            .map(|i| {
                let mut row = Vec::with_capacity(size.saturating_sub(i + 1));
                let first = self.candidates[i];
                for j in (i + 1)..size {
                    let second = self.candidates[j];
                    row.push((i, j, self.coarse_best_delta_e(first, second)));
                }
                row
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
            .collect();

        coarse.sort_by(|a, b| a.2.total_cmp(&b.2));
        coarse.truncate(Self::PAIRS_KEPT);

        let ratios = &*PAIR_RATIOS;
        coarse
            .into_par_iter()
            .map(|(i, j, _)| self.refine(&[self.candidates[i], self.candidates[j]], ratios))
            .collect()
    }

    fn coarse_best_delta_e(&self, first: Candidate<'a>, second: Candidate<'a>) -> f64 {
        let pair = [first, second];
        let mut best_de = f64::INFINITY;
        for weight in Self::COARSE_WEIGHTS {
            let color = Self::blend(&pair, &[weight, 1.0 - weight]);
            let delta_e = delta_e_2000_lab(self.target, color.to_lab());
            if delta_e < best_de {
                best_de = delta_e;
            }
        }
        best_de
    }

    fn combinations(&self, count: usize) -> Vec<Evaluation<'a>> {
        let pool = self.pool_for(count);
        let Some(ratios) = RATIO_GRIDS.get(&count) else {
            return Vec::new();
        };
        if pool.len() < count {
            return Vec::new();
        }

        let mut picks = Vec::new();
        let mut current = vec![0; count];
        Self::combine(pool.len(), count, 0, 0, &mut current, &mut picks);

        picks
            .into_par_iter()
            .map(|indexes| {
                let chosen: Vec<Candidate<'a>> = indexes.into_iter().map(|i| pool[i]).collect();
                self.refine(&chosen, ratios)
            })
            .collect()
    }

    fn pool_for(&self, count: usize) -> Vec<Candidate<'a>> {
        let nearest = match count {
            3 => 18,
            4 => 12,
            _ => 9,
        };

        let mut by_single: Vec<(usize, f64)> = self
            .candidates
            .iter()
            .enumerate()
            .map(|(idx, &c)| (idx, self.evaluate(&[c], &[1]).delta_e))
            .collect();
        by_single.sort_by(|a, b| a.1.total_cmp(&b.1));

        let mut pool_indices = IndexSet::new();
        for (idx, _) in by_single.into_iter().take(nearest) {
            pool_indices.insert(idx);
        }

        // maxByOrNull / minByOrNull in Kotlin keeps the first element encountered on ties
        if let Some((max_l_idx, _)) =
            self.candidates
                .iter()
                .enumerate()
                .fold(None, |acc: Option<(usize, f64)>, (i, c)| match acc {
                    None => Some((i, c.lab.l)),
                    Some((_, best_l)) if c.lab.l > best_l => Some((i, c.lab.l)),
                    other => other,
                })
        {
            pool_indices.insert(max_l_idx);
        }

        if let Some((min_l_idx, _)) =
            self.candidates
                .iter()
                .enumerate()
                .fold(None, |acc: Option<(usize, f64)>, (i, c)| match acc {
                    None => Some((i, c.lab.l)),
                    Some((_, best_l)) if c.lab.l < best_l => Some((i, c.lab.l)),
                    other => other,
                })
        {
            pool_indices.insert(min_l_idx);
        }

        let mut by_tinting: Vec<(usize, f64)> = self
            .candidates
            .iter()
            .enumerate()
            .map(|(idx, c)| (idx, c.tinting))
            .collect();
        by_tinting.sort_by(|a, b| b.1.total_cmp(&a.1));
        for (idx, _) in by_tinting.into_iter().take(Self::STRUCTURAL_PAINTS) {
            pool_indices.insert(idx);
        }

        for sector in 0..Self::HUE_SECTORS {
            let from = (sector as f64) * 360.0 / (Self::HUE_SECTORS as f64);
            let to = ((sector + 1) as f64) * 360.0 / (Self::HUE_SECTORS as f64);
            let best_in_sector = self
                .candidates
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    let h = c.hue();
                    h >= from && h < to
                })
                .fold(None, |acc: Option<(usize, f64)>, (i, c)| {
                    let ch = c.chroma();
                    match acc {
                        None => Some((i, ch)),
                        Some((_, best_ch)) if ch > best_ch => Some((i, ch)),
                        other => other,
                    }
                });
            if let Some((idx, _)) = best_in_sector {
                pool_indices.insert(idx);
            }
        }

        pool_indices
            .into_iter()
            .map(|idx| self.candidates[idx])
            .collect()
    }

    fn refine(&self, paints: &[Candidate<'a>], ratios: &[Vec<i32>]) -> Evaluation<'a> {
        let mut best_parts: Option<&[i32]> = None;
        let mut best_color = Rgb::new(0.0, 0.0, 0.0);
        let mut best_de = f64::INFINITY;
        let mut weights = [0.0; Self::MAX_PAINTS];

        for parts in ratios {
            for (i, (paint, &part)) in paints.iter().zip(parts.iter()).enumerate() {
                weights[i] = (part as f64) * paint.tinting;
            }
            let color = Self::blend(paints, &weights[..paints.len()]);
            let delta_e = delta_e_2000_lab(self.target, color.to_lab());
            if best_parts.is_none() || delta_e < best_de {
                best_parts = Some(parts);
                best_color = color;
                best_de = delta_e;
            }
        }

        Evaluation {
            paints: paints.to_vec(),
            parts: best_parts.expect("ratios non vide").to_vec(),
            color: best_color,
            delta_e: best_de,
        }
    }

    fn evaluate(&self, paints: &[Candidate<'a>], parts: &[i32]) -> Evaluation<'a> {
        let mut weights = [0.0; Self::MAX_PAINTS];
        for (i, (paint, &part)) in paints.iter().zip(parts.iter()).enumerate() {
            weights[i] = (part as f64) * paint.tinting;
        }
        let color = Self::blend(paints, &weights[..paints.len()]);
        let delta_e = delta_e_2000_lab(self.target, color.to_lab());
        Evaluation {
            paints: paints.to_vec(),
            parts: parts.to_vec(),
            color,
            delta_e,
        }
    }

    fn blend(paints: &[Candidate<'a>], weights: &[f64]) -> Rgb {
        let total: f64 = weights.iter().sum();
        let mut absorption = [0.0; 3];
        let mut scattering = [0.0; 3];
        for (candidate, &weight) in paints.iter().zip(weights.iter()) {
            let share = weight / total;
            for channel in 0..3 {
                absorption[channel] += share * candidate.absorption[channel];
                scattering[channel] += share * candidate.scattering[channel];
            }
        }
        Colorant::color_of(absorption, scattering)
    }

    fn combine(
        size: usize,
        count: usize,
        filled: usize,
        start: usize,
        current: &mut [usize],
        out: &mut Vec<Vec<usize>>,
    ) {
        if filled == count {
            out.push(current.to_vec());
            return;
        }
        for i in start..size {
            current[filled] = i;
            Self::combine(size, count, filled + 1, i + 1, current, out);
        }
    }

    fn best(&self, evaluations: Vec<Evaluation<'a>>, max_results: usize) -> Vec<MixSuggestion> {
        let mut by_set: IndexMap<String, Evaluation<'a>> = IndexMap::new();
        for evaluation in evaluations {
            let mut names: Vec<String> = evaluation
                .paints
                .iter()
                .map(|c| c.paint.display_name())
                .collect();
            names.sort();
            let key = format!("|{}", names.join("|"));
            match by_set.get(&key) {
                Some(prev) if prev.delta_e <= evaluation.delta_e => {}
                _ => {
                    by_set.insert(key, evaluation);
                }
            }
        }

        let mut ranked: Vec<Evaluation<'a>> = by_set.into_values().collect();
        ranked.sort_by(Self::cmp_quality_then_simplicity);

        let mut distinct: Vec<Evaluation<'a>> = Vec::new();
        for evaluation in ranked {
            let already_covered = distinct
                .iter()
                .any(|d| delta_e_2000(d.color, evaluation.color) < Self::DUPLICATE_RESULT);
            if !already_covered {
                distinct.push(evaluation);
            }
            if distinct.len() == max_results {
                break;
            }
        }

        distinct
            .into_iter()
            .map(|eval| MixSuggestion {
                parts: eval
                    .paints
                    .into_iter()
                    .zip(eval.parts)
                    .map(|(c, part)| PaintPart::of(c.paint.clone(), part as f64))
                    .collect(),
                color: eval.color,
                delta_e: eval.delta_e,
            })
            .collect()
    }

    fn cmp_quality_then_simplicity(a: &Evaluation<'a>, b: &Evaluation<'a>) -> Ordering {
        let tier_a = (a.delta_e / Self::PERCEPTUAL_TIE) as i32;
        let tier_b = (b.delta_e / Self::PERCEPTUAL_TIE) as i32;
        tier_a
            .cmp(&tier_b)
            .then_with(|| a.paints.len().cmp(&b.paints.len()))
            .then_with(|| {
                let sum_a: i32 = a.parts.iter().sum();
                let sum_b: i32 = b.parts.iter().sum();
                sum_a.cmp(&sum_b)
            })
            .then_with(|| a.delta_e.total_cmp(&b.delta_e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::CatalogLoader;
    use std::time::Instant;

    const TARGETS: [&str; 10] = [
        "#C98F72", "#8A4A3C", "#E0B49A", "#6B5A42", "#4A5560", "#9A8F80", "#3E4A3A", "#B29B62",
        "#7A3A22", "#2C3E50",
    ];

    fn load_catalogue() -> Vec<Paint> {
        CatalogLoader::default()
            .load()
            .into_iter()
            .filter(|p| p.system == PaintSystem::Oil)
            .collect()
    }

    fn find_wn(catalog: &[Paint], name: &str) -> Paint {
        catalog
            .iter()
            .find(|p| p.brand == "Winsor & Newton" && p.name == name)
            .unwrap()
            .clone()
    }

    #[test]
    fn catalog_color_is_found_identically_by_single_tube() {
        let service = ColorMixService::new();
        let catalog = load_catalogue();
        let reference = catalog.iter().find(|p| p.name == "Burnt Umber").unwrap();

        let best = &service.suggest_mixes(reference.color(), &catalog, 1, None)[0];
        assert!(best.delta_e < 0.5, "écart {}", best.delta_e);
        assert_eq!(1, best.parts.len());
    }

    #[test]
    fn each_suggestion_reports_exact_delta_e_of_proposed_ratio() {
        let service = ColorMixService::new();
        let catalog = load_catalogue();
        for hex in TARGETS {
            let target = Rgb::of_hex(hex).unwrap();
            for suggestion in service.suggest_mixes(target, &catalog, 5, None) {
                let actual = service.mix(&suggestion.parts).unwrap().color;
                assert!(
                    (delta_e_2000(target, actual) - suggestion.delta_e).abs() < 0.01,
                    "{hex} : écart annoncé pour {}",
                    suggestion.describe()
                );
            }
        }
    }

    #[test]
    fn proposed_ratios_are_simple_integers() {
        let service = ColorMixService::new();
        let catalog = load_catalogue();
        for hex in TARGETS {
            let target = Rgb::of_hex(hex).unwrap();
            for suggestion in service.suggest_mixes(target, &catalog, 5, None) {
                for part in suggestion.parts {
                    assert_eq!(part.parts.round(), part.parts);
                    assert!((1.0..=30.0).contains(&part.parts));
                }
            }
        }
    }

    #[test]
    fn requested_max_paints_is_respected() {
        let service = ColorMixService::new();
        let catalog = load_catalogue();
        for hex in TARGETS {
            let target = Rgb::of_hex(hex).unwrap();
            for max in 1..=5 {
                for suggestion in service.suggest_mixes(target, &catalog, 8, Some(max)) {
                    assert!(suggestion.parts.len() <= max, "{hex} avec {max} tubes");
                }
            }
        }
    }

    #[test]
    fn short_palette_benefits_from_extra_tubes() {
        let service = ColorMixService::new();
        let catalog = load_catalogue();
        let short_palette = vec![
            find_wn(&catalog, "Cadmium Yellow Pale"),
            find_wn(&catalog, "Permanent Rose"),
            find_wn(&catalog, "Winsor Blue (Green Shade)"),
            find_wn(&catalog, "Titanium White"),
            find_wn(&catalog, "Burnt Umber"),
        ];

        let mut with_three = 0.0;
        let mut with_four = 0.0;
        for hex in TARGETS {
            let target = Rgb::of_hex(hex).unwrap();
            with_three += service.suggest_mixes(target, &short_palette, 1, Some(3))[0].delta_e;
            with_four += service.suggest_mixes(target, &short_palette, 1, Some(4))[0].delta_e;
        }
        assert!(
            with_four < with_three,
            "trois tubes : {with_three}, quatre tubes : {with_four}"
        );
    }

    #[test]
    fn list_proposes_distinct_tube_combinations() {
        let service = ColorMixService::new();
        let catalog = load_catalogue();
        let suggestions = service.suggest_mixes(Rgb::of_hex("#C98F72").unwrap(), &catalog, 8, None);

        let combinations: Vec<Vec<String>> = suggestions
            .iter()
            .map(|s| {
                let mut names: Vec<String> =
                    s.parts.iter().map(|p| p.paint.display_name()).collect();
                names.sort();
                names
            })
            .collect();
        let unique: IndexSet<Vec<String>> = combinations.iter().cloned().collect();
        assert_eq!(combinations.len(), unique.len());
    }

    #[test]
    fn allowing_one_more_tube_never_degrades_result() {
        let service = ColorMixService::new();
        let catalog = load_catalogue();
        for hex in TARGETS {
            let target = Rgb::of_hex(hex).unwrap();
            let mut previous = f64::MAX;
            for max in 1..=5 {
                let current = service.suggest_mixes(target, &catalog, 1, Some(max))[0].delta_e;
                assert!(current <= previous + 1e-9, "cible {hex} avec {max} tubes");
                previous = current;
            }
        }
    }

    #[test]
    fn suggestions_are_ranked_by_perceptual_tier() {
        let service = ColorMixService::new();
        let catalog = load_catalogue();
        let suggestions = service.suggest_mixes(Rgb::of_hex("#6B5A42").unwrap(), &catalog, 8, None);
        let tiers: Vec<i32> = suggestions
            .iter()
            .map(|s| (s.delta_e / 0.5) as i32)
            .collect();
        let mut sorted = tiers.clone();
        sorted.sort();
        assert_eq!(sorted, tiers);
    }

    #[test]
    fn within_imperceptible_gap_simplest_mix_wins() {
        let service = ColorMixService::new();
        let catalog = load_catalogue();
        let reference = catalog.iter().find(|p| p.name == "Yellow Ochre").unwrap();
        let best = &service.suggest_mixes(reference.color(), &catalog, 5, None)[0];
        assert_eq!(1, best.parts.len(), "{}", best.describe());
        assert!(best.delta_e < 0.5);
    }

    #[test]
    fn oil_and_acrylic_never_end_up_in_the_same_mix() {
        let service = ColorMixService::new();
        let all = CatalogLoader::default().load();
        assert!(all.iter().any(|p| p.system == PaintSystem::Acrylic));
        let oil = all
            .iter()
            .find(|p| p.system == PaintSystem::Oil)
            .unwrap()
            .clone();
        let acr = all
            .iter()
            .find(|p| p.system == PaintSystem::Acrylic)
            .unwrap()
            .clone();
        assert!(
            service
                .mix(&[PaintPart::of(oil, 1.0), PaintPart::of(acr, 1.0)])
                .is_err()
        );

        let candidates: Vec<Paint> = all
            .iter()
            .take(40)
            .chain(all.iter().rev().take(40))
            .cloned()
            .collect();
        for s in service.suggest_mixes(Rgb::of_hex("#6A7F4C").unwrap(), &candidates, 6, Some(3)) {
            let first = s.parts[0].paint.system;
            assert!(
                s.parts.iter().all(|p| p.paint.system == first),
                "{}",
                s.describe()
            );
        }
    }

    #[test]
    fn recommended_max_paints_adapts_to_selection_size() {
        assert_eq!(5, ColorMixService::recommended_max_paints(4));
        assert_eq!(5, ColorMixService::recommended_max_paints(8));
        assert_eq!(4, ColorMixService::recommended_max_paints(9));
        assert_eq!(4, ColorMixService::recommended_max_paints(24));
        assert_eq!(3, ColorMixService::recommended_max_paints(25));
        assert_eq!(3, ColorMixService::recommended_max_paints(439));
    }

    #[test]
    fn search_cost_and_worst_delta_e_on_full_catalogue() {
        let service = ColorMixService::new();
        let catalog = load_catalogue();
        let start = Instant::now();
        let mut worst: f64 = 0.0;
        for hex in TARGETS {
            let target = Rgb::of_hex(hex).unwrap();
            let best_de = service.suggest_mixes(target, &catalog, 8, None)[0].delta_e;
            worst = worst.max(best_de);
        }
        let millis = start.elapsed().as_millis() / (TARGETS.len() as u128);
        println!(
            "recherche Rust : {} tubes, {} ms par cible, pire écart {}",
            catalog.len(),
            millis,
            worst
        );
        assert!(millis < 1000, "{millis} ms par cible");
        assert!(
            (worst - 0.8537644933880194).abs() < 1e-12,
            "pire écart attendu 0.8537644933880194, obtenu {worst}"
        );
    }
}
