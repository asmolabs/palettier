use crate::drying::{DryingContext, DryingEstimate, DryingTimeService, waits_for_through_dry};
use crate::i18n::Locale;
use crate::paint::{DryingClass, PaintSystem};
use crate::project::{Project, ProjectLayer, ProjectZone};
use chrono::{DateTime, Utc};
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::time::Duration;

/// Drying stage of an applied coat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Stage {
    Open,
    Setting,
    TouchDry,
    Recoatable,
    ThroughDry,
    Cured,
}

impl Stage {
    pub fn label(&self) -> &'static str {
        match self {
            Stage::Open => "Encore ouverte",
            Stage::Setting => "En prise",
            Stage::TouchDry => "Seche au toucher",
            Stage::Recoatable => "Recouvrable",
            Stage::ThroughDry => "Seche a coeur",
            Stage::Cured => "Polymerisee",
        }
    }

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.label(),
            Locale::En => match self {
                Stage::Open => "Still open",
                Stage::Setting => "Setting",
                Stage::TouchDry => "Touch dry",
                Stage::Recoatable => "Recoatable",
                Stage::ThroughDry => "Through dry",
                Stage::Cured => "Cured",
            },
        }
    }

    pub fn meaning(&self) -> &'static str {
        match self {
            Stage::Open => "Fondus et reprises dans le frais sont encore possibles.",
            Stage::Setting => "Elle ne se travaille plus sans l'arracher : ne la touchez pas.",
            Stage::TouchDry => {
                "Elle ne marque plus, mais une couche posee dessus la releverait encore."
            }
            Stage::Recoatable => "La couche suivante peut etre posee sans relever celle-ci.",
            Stage::ThroughDry => "Manipulation, masquage et montage possibles.",
            Stage::Cured => "Vernis final possible.",
        }
    }

    pub fn meaning_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.meaning(),
            Locale::En => match self {
                Stage::Open => "Wet-in-wet blending and rework are still possible.",
                Stage::Setting => "Working it now will tear the film: leave it alone.",
                Stage::TouchDry => {
                    "Dry to the touch, but applying a new coat on top would still lift it."
                }
                Stage::Recoatable => "The next coat can be applied without lifting this one.",
                Stage::ThroughDry => "Safe to handle, mask, and assemble.",
                Stage::Cured => "Ready for final varnish.",
            },
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// An applied coat evaluated at the current time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoatState {
    pub zone: String,
    pub role: String,
    pub target_hex: String,
    pub applied_at: DateTime<Utc>,
    pub drying_class: DryingClass,
    pub estimate: DryingEstimate,
    pub stage: Stage,
    pub next_role: Option<String>,
    pub until_next: Duration,
}

impl CoatState {
    pub fn ready_for_next(&self) -> bool {
        self.until_next.is_zero()
    }
}

/// A zone and its painting progress.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZoneState {
    pub name: String,
    pub material: String,
    pub applied: usize,
    pub total: usize,
    pub last: Option<CoatState>,
}

impl ZoneState {
    pub fn done(&self) -> bool {
        self.total > 0 && self.applied == self.total
    }

    pub fn not_started(&self) -> bool {
        self.applied == 0
    }

    pub fn ready_now(&self) -> bool {
        !self.done() && self.last.as_ref().is_none_or(|c| c.ready_for_next())
    }

    pub fn remaining(&self) -> Duration {
        if self.ready_now() || self.done() {
            Duration::ZERO
        } else {
            self.last
                .as_ref()
                .map(|c| c.until_next)
                .unwrap_or(Duration::ZERO)
        }
    }
}

/// A piece in progress and the state of each of its zones.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PieceState {
    pub project: Project,
    pub zones: Vec<ZoneState>,
}

impl PieceState {
    pub fn ready_zones(&self) -> Vec<&ZoneState> {
        self.zones.iter().filter(|z| z.ready_now()).collect()
    }

    pub fn ready_now(&self) -> bool {
        self.zones.iter().any(|z| z.ready_now())
    }

    pub fn done(&self) -> bool {
        !self.zones.is_empty() && self.zones.iter().all(|z| z.done())
    }

    pub fn next_availability(&self) -> Option<Duration> {
        if self.ready_now() || self.done() {
            None
        } else {
            self.zones
                .iter()
                .filter(|z| !z.done())
                .map(|z| z.remaining())
                .min()
        }
    }

    pub fn applied_coats(&self) -> usize {
        self.zones.iter().map(|z| z.applied).sum()
    }

    pub fn total_coats(&self) -> usize {
        self.zones.iter().map(|z| z.total).sum()
    }
}

/// The full workbench, with ready-to-paint pieces ordered first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bench {
    pub pieces: Vec<PieceState>,
}

impl Bench {
    pub fn ready(&self) -> Vec<&PieceState> {
        self.pieces.iter().filter(|p| p.ready_now()).collect()
    }

    pub fn waiting(&self) -> Vec<&PieceState> {
        self.pieces
            .iter()
            .filter(|p| !p.ready_now() && !p.done())
            .collect()
    }

    pub fn done(&self) -> Vec<&PieceState> {
        self.pieces.iter().filter(|p| p.done()).collect()
    }

    pub fn next_availability(&self) -> Option<Duration> {
        self.waiting()
            .into_iter()
            .filter_map(|p| p.next_availability())
            .min()
    }
}

/// Workbench service: tracks pieces in progress and determines which zones are ready now.
#[derive(Debug, Clone, Default)]
pub struct WorkbenchService {
    drying_time_service: DryingTimeService,
}

impl WorkbenchService {
    pub fn new(drying_time_service: DryingTimeService) -> Self {
        Self {
            drying_time_service,
        }
    }

    pub fn bench(&self, projects: &[Project], now: DateTime<Utc>) -> Bench {
        let mut pieces: Vec<PieceState> = projects
            .iter()
            .map(|p| self.state_project(p, now))
            .collect();
        pieces.sort_by(Self::cmp_urgency);
        Bench { pieces }
    }

    fn state_project(&self, project: &Project, now: DateTime<Utc>) -> PieceState {
        let zones = project
            .zones
            .iter()
            .map(|z| self.state_zone(z, project.system, now))
            .collect();
        PieceState {
            project: project.clone(),
            zones,
        }
    }

    fn state_zone(&self, zone: &ProjectZone, system: PaintSystem, now: DateTime<Utc>) -> ZoneState {
        let applied_count = zone.layers.iter().filter(|l| l.is_applied()).count();
        let last_idx = zone
            .layers
            .iter()
            .enumerate()
            .filter_map(|(idx, l)| l.applied.as_ref().map(|a| (idx, a.at)))
            .max_by_key(|(_, at)| *at)
            .map(|(idx, _)| idx);

        let next = Self::next_after(&zone.layers, last_idx);
        let last_coat = last_idx.map(|idx| self.coat(zone, &zone.layers[idx], next, system, now));

        ZoneState {
            name: zone.name.clone(),
            material: zone.material.clone(),
            applied: applied_count,
            total: zone.layers.len(),
            last: last_coat,
        }
    }

    fn next_after(layers: &[ProjectLayer], last_idx: Option<usize>) -> Option<&ProjectLayer> {
        let from = last_idx.map_or(0, |i| i + 1);
        for layer in &layers[from..] {
            if !layer.is_applied() {
                return Some(layer);
            }
        }
        layers.iter().find(|l| !l.is_applied())
    }

    fn coat(
        &self,
        zone: &ProjectZone,
        layer: &ProjectLayer,
        next: Option<&ProjectLayer>,
        system: PaintSystem,
        now: DateTime<Utc>,
    ) -> CoatState {
        let applied = layer.applied.as_ref().expect("couche posée");
        let technique = layer.technique;
        let estimate = self
            .drying_time_service
            .estimate(&DryingContext::for_technique_in(
                technique,
                system,
                applied.drying_class,
                applied.workshop,
            ));

        let elapsed = (now - applied.at).to_std().unwrap_or(Duration::ZERO);
        let gate = match next {
            None => Duration::ZERO,
            Some(n) => {
                if waits_for_through_dry(
                    layer.system_within(system),
                    n.system_within(system),
                    n.technique,
                ) {
                    estimate.through_dry
                } else {
                    estimate.recoat
                }
            }
        };

        CoatState {
            zone: zone.name.clone(),
            role: layer.role.clone(),
            target_hex: layer.target_hex.clone(),
            applied_at: applied.at,
            drying_class: applied.drying_class,
            stage: Self::stage_of(elapsed, &estimate),
            next_role: next.map(|n| n.role.clone()),
            until_next: gate.saturating_sub(elapsed),
            estimate,
        }
    }

    fn cmp_urgency(a: &PieceState, b: &PieceState) -> Ordering {
        let rank = |p: &PieceState| {
            if p.ready_now() {
                0
            } else if p.done() {
                2
            } else {
                1
            }
        };
        rank(a).cmp(&rank(b)).then_with(|| {
            let wait_a = a.next_availability().unwrap_or(Duration::ZERO);
            let wait_b = b.next_availability().unwrap_or(Duration::ZERO);
            wait_a.cmp(&wait_b)
        })
    }

    fn stage_of(elapsed: Duration, estimate: &DryingEstimate) -> Stage {
        let milestones = [
            (Stage::Open, estimate.open_time),
            (Stage::Setting, estimate.touch_dry),
            (Stage::TouchDry, estimate.recoat),
            (Stage::Recoatable, estimate.through_dry),
            (Stage::ThroughDry, estimate.full_cure),
        ];
        let mut previous = Duration::ZERO;
        for (stage, milestone) in milestones {
            let bound = milestone.max(previous);
            if elapsed < bound {
                return stage;
            }
            previous = bound;
        }
        Stage::Cured
    }
}

/// Tracks zones that have newly become ready since the last check.
#[derive(Debug, Clone, Default)]
pub struct ReadinessWatch {
    ready: IndexSet<String>,
    primed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreedZone {
    pub piece: String,
    pub zone: String,
}

impl FreedZone {
    pub fn label(&self) -> String {
        format!("{} - {}", self.piece, self.zone)
    }
}

impl ReadinessWatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn newly_ready(&mut self, bench: &Bench) -> Vec<FreedZone> {
        let mut now = IndexSet::new();
        let mut freed = Vec::new();

        for piece in &bench.pieces {
            for zone in piece.ready_zones() {
                let id_str = piece
                    .project
                    .id
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "null".to_string());
                let key = format!("{id_str}/{}", zone.name);
                now.insert(key.clone());
                if self.primed && !self.ready.contains(&key) {
                    freed.push(FreedZone {
                        piece: piece.project.name.clone(),
                        zone: zone.name.clone(),
                    });
                }
            }
        }

        self.ready = now;
        let first = !self.primed;
        self.primed = true;
        if first { Vec::new() } else { freed }
    }

    pub fn reset(&mut self) {
        self.ready.clear();
        self.primed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drying::Workshop;
    use crate::paint::Ventilation;
    use chrono::TimeDelta;

    fn make_layer(role: &str) -> ProjectLayer {
        let mut l = ProjectLayer::new(role, "#C98F72");
        l.technique = crate::paint::Technique::Glaze;
        l
    }

    fn make_piece(name: &str, id: i64) -> Project {
        let mut p = Project::new(name);
        p.id = Some(id);
        p.subject = "Buste".to_string();
        p.zones = vec![
            ProjectZone {
                id: None,
                name: "Visage".to_string(),
                material: "Peau".to_string(),
                note: String::new(),
                layers: vec![
                    make_layer("Ombre 1"),
                    make_layer("Base"),
                    make_layer("Lumiere 1"),
                ],
            },
            ProjectZone {
                id: None,
                name: "Cape".to_string(),
                material: "Tissu".to_string(),
                note: String::new(),
                layers: vec![
                    make_layer("Ombre 1"),
                    make_layer("Base"),
                    make_layer("Lumiere 1"),
                ],
            },
        ];
        p
    }

    fn apply_layer(
        mut project: Project,
        zone_index: usize,
        layer_index: usize,
        at: DateTime<Utc>,
        workshop: Workshop,
        drying: DryingClass,
    ) -> Project {
        project.zones[zone_index].layers[layer_index] =
            project.zones[zone_index].layers[layer_index].mark_applied(at, workshop, drying);
        project
    }

    #[test]
    fn unpainted_piece_is_available_immediately() {
        let now = "2026-03-01T10:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let service = WorkbenchService::default();
        let piece = &service.bench(&[make_piece("Rien de pose", 1)], now).pieces[0];

        assert!(piece.zones.iter().all(|z| z.not_started()));
        assert!(piece.ready_now());
        assert!(piece.next_availability().is_none());
        assert_eq!(0, piece.applied_coats());
        assert_eq!(6, piece.total_coats());
    }

    #[test]
    fn fresh_layer_puts_its_zone_on_hold_without_blocking_others() {
        let now = "2026-03-01T10:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let service = WorkbenchService::default();
        let project = apply_layer(
            make_piece("Visage frais", 1),
            0,
            0,
            now,
            Workshop::standard(),
            DryingClass::Slow,
        );
        let piece = &service
            .bench(&[project], now + TimeDelta::minutes(30))
            .pieces[0];

        let painted = &piece.zones[0];
        let untouched = &piece.zones[1];

        assert!(!painted.ready_now());
        assert_eq!(Stage::Open, painted.last.as_ref().unwrap().stage);
        assert!(!painted.remaining().is_zero());
        assert!(untouched.ready_now());
        assert!(piece.ready_now());
    }

    #[test]
    fn after_delay_zone_becomes_available_again() {
        let now = "2026-03-01T10:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let service = WorkbenchService::default();
        let mut project = apply_layer(
            make_piece("Visage sec", 1),
            0,
            0,
            now,
            Workshop::standard(),
            DryingClass::Fast,
        );
        project = apply_layer(project, 1, 0, now, Workshop::standard(), DryingClass::Fast);

        let wet = &service
            .bench(std::slice::from_ref(&project), now + TimeDelta::hours(2))
            .pieces[0];
        assert!(!wet.ready_now());
        assert!(wet.next_availability().is_some());

        let dry = &service.bench(&[project], now + TimeDelta::days(30)).pieces[0];
        assert!(dry.zones.iter().all(|z| z.ready_now()));
        assert_eq!(Stage::Cured, dry.zones[0].last.as_ref().unwrap().stage);
        assert!(dry.next_availability().is_none());
    }

    #[test]
    fn drying_uses_workshop_conditions_from_application_time() {
        let now = "2026-03-01T10:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let service = WorkbenchService::default();
        let cold = apply_layer(
            make_piece("Atelier froid", 1),
            0,
            0,
            now,
            Workshop {
                temperature_celsius: 10.0,
                relative_humidity: 80.0,
                ventilation: Ventilation::Confined,
            },
            DryingClass::Medium,
        );
        let warm = apply_layer(
            make_piece("Atelier chaud", 2),
            0,
            0,
            now,
            Workshop {
                temperature_celsius: 28.0,
                relative_humidity: 35.0,
                ventilation: Ventilation::Good,
            },
            DryingClass::Medium,
        );

        let later = now + TimeDelta::days(2);
        let cold_rem = service.bench(&[cold], later).pieces[0].zones[0].remaining();
        let warm_rem = service.bench(&[warm], later).pieces[0].zones[0].remaining();
        assert!(cold_rem > warm_rem);
    }

    #[test]
    fn painting_out_of_order_does_not_announce_already_covered_layer() {
        let now = "2026-03-01T10:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let later = now + TimeDelta::days(30);
        let service = WorkbenchService::default();

        let after_base = apply_layer(
            make_piece("Base d'abord", 1),
            0,
            1,
            now,
            Workshop::standard(),
            DryingClass::Fast,
        );
        let last_base = service.bench(&[after_base], later).pieces[0].zones[0]
            .last
            .clone()
            .unwrap();
        assert_eq!("Base", last_base.role);
        assert_eq!(Some("Lumiere 1".to_string()), last_base.next_role);

        let after_hl = apply_layer(
            make_piece("Lumiere d'abord", 1),
            0,
            2,
            now,
            Workshop::standard(),
            DryingClass::Fast,
        );
        let last_hl = service.bench(&[after_hl], later).pieces[0].zones[0]
            .last
            .clone()
            .unwrap();
        assert_eq!("Lumiere 1", last_hl.role);
        assert_eq!(Some("Ombre 1".to_string()), last_hl.next_role);
    }
}
