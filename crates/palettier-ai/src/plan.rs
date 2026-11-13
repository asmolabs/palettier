use crate::engine::{ChatEngine, ImageAttachment};
use palettier_domain::color::Rgb;
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::i18n::Locale;
use palettier_domain::mix::ColorMixService;
use palettier_domain::paint::{Paint, PaintSystem, Technique};
use palettier_domain::plan::{PaintingPlan, PlanLayer, PlanZone};
use palettier_image::ImageDecoder;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::{Arc, LazyLock};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDraft {
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub general_approach: String,
    #[serde(default)]
    pub zones: Vec<ZoneDraft>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneDraft {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub material: String,
    #[serde(default)]
    pub shadow1: Option<LayerDraft>,
    #[serde(default)]
    pub shadow2: Option<LayerDraft>,
    #[serde(default)]
    pub base: Option<LayerDraft>,
    #[serde(default)]
    pub highlight1: Option<LayerDraft>,
    #[serde(default)]
    pub highlight2: Option<LayerDraft>,
    #[serde(default)]
    pub accents: Vec<LayerDraft>,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerDraft {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub target_hex: String,
    #[serde(default)]
    pub technique: String,
    #[serde(default)]
    pub note: String,
}

pub static PLAN_SCHEMA: LazyLock<Value> = LazyLock::new(|| {
    let layer = json!({
        "type": "object",
        "properties": {
            "role": { "type": "string" },
            "targetHex": { "type": "string", "pattern": "^#[0-9A-Fa-f]{6}$" },
            "technique": { "type": "string" },
            "note": { "type": "string" }
        },
        "required": ["role", "targetHex", "technique", "note"]
    });
    json!({
        "type": "object",
        "properties": {
            "subject": { "type": "string" },
            "generalApproach": { "type": "string" },
            "zones": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "material": { "type": "string" },
                        "shadow1": layer.clone(),
                        "shadow2": layer.clone(),
                        "base": layer.clone(),
                        "highlight1": layer.clone(),
                        "highlight2": layer.clone(),
                        "accents": { "type": "array", "items": layer },
                        "note": { "type": "string" }
                    },
                    "required": [
                        "name", "material",
                        "shadow1", "shadow2", "base", "highlight1", "highlight2",
                        "accents", "note"
                    ]
                }
            },
            "warnings": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["subject", "generalApproach", "zones", "warnings"]
    })
});

#[derive(Default, Clone)]
pub struct PlanEnricher {
    mix_service: ColorMixService,
}

impl PlanEnricher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enrich(&self, draft: &PlanDraft, palette_name: &str, palette: &[Paint]) -> PaintingPlan {
        self.enrich_i18n(draft, palette_name, palette, Locale::Fr)
    }

    pub fn enrich_i18n(
        &self,
        draft: &PlanDraft,
        palette_name: &str,
        palette: &[Paint],
        locale: Locale,
    ) -> PaintingPlan {
        let system = Self::system_of(palette);
        let zones = draft
            .zones
            .iter()
            .map(|z| self.enrich_zone(z, palette, locale))
            .collect();
        PaintingPlan {
            system,
            subject: if draft.subject.trim().is_empty() {
                match locale {
                    Locale::En => "Unspecified subject".to_string(),
                    Locale::Fr => "Sujet non précisé".to_string(),
                }
            } else {
                draft.subject.clone()
            },
            palette_name: palette_name.to_string(),
            approach: draft.general_approach.clone(),
            zones,
        }
    }

    /// The system the plan is painted in: that of most of the palette's paints.
    pub fn system_of(palette: &[Paint]) -> PaintSystem {
        let acrylic = palette
            .iter()
            .filter(|p| p.system == PaintSystem::Acrylic)
            .count();
        if acrylic * 2 > palette.len() {
            PaintSystem::Acrylic
        } else {
            PaintSystem::Oil
        }
    }

    fn enrich_zone(&self, zone: &ZoneDraft, palette: &[Paint], locale: Locale) -> PlanZone {
        let fallback_shadow = Rgb::new(0.23, 0.16, 0.13);
        let fallback_base = Rgb::new(0.55, 0.42, 0.34);
        let fallback_light = Rgb::new(0.85, 0.75, 0.65);

        let base_rgb = Self::parse_or(zone.base.as_ref(), fallback_base);
        let shadow1_rgb = Self::parse_or(zone.shadow1.as_ref(), fallback_shadow);
        let highlight2_rgb = Self::parse_or(zone.highlight2.as_ref(), fallback_light);
        let shadow2_rgb =
            Self::parse_or(zone.shadow2.as_ref(), Self::midpoint(shadow1_rgb, base_rgb));
        let highlight1_rgb = Self::parse_or(
            zone.highlight1.as_ref(),
            Self::midpoint(base_rgb, highlight2_rgb),
        );

        let (role_s2, role_s1, role_base, role_h1, role_h2) = match locale {
            Locale::En => ("Shadow 2", "Shadow 1", "Base", "Highlight 1", "Highlight 2"),
            Locale::Fr => ("Ombre 2", "Ombre 1", "Base", "Lumière 1", "Lumière 2"),
        };

        // `PlanZone::layers()` expects `shadows` in `[Shadow 2, Shadow 1]` order (since it iterates with `shadows.iter().rev()`)
        let shadows = vec![
            self.to_layer(zone.shadow2.as_ref(), role_s2, shadow2_rgb, palette, locale),
            self.to_layer(zone.shadow1.as_ref(), role_s1, shadow1_rgb, palette, locale),
        ];
        let base = Some(self.to_layer(zone.base.as_ref(), role_base, base_rgb, palette, locale));
        let highlights = vec![
            self.to_layer(
                zone.highlight1.as_ref(),
                role_h1,
                highlight1_rgb,
                palette,
                locale,
            ),
            self.to_layer(
                zone.highlight2.as_ref(),
                role_h2,
                highlight2_rgb,
                palette,
                locale,
            ),
        ];

        let accents = zone
            .accents
            .iter()
            .map(|accent| {
                let color = Self::parse_or(Some(accent), base_rgb);
                let role = if accent.role.trim().is_empty() {
                    "Variation"
                } else {
                    &accent.role
                };
                self.to_layer(Some(accent), role, color, palette, locale)
            })
            .collect();

        PlanZone {
            name: if zone.name.trim().is_empty() {
                "Zone".to_string()
            } else {
                zone.name.clone()
            },
            material: zone.material.clone(),
            note: zone.note.clone(),
            base,
            shadows,
            highlights,
            accents,
        }
    }

    fn to_layer(
        &self,
        draft: Option<&LayerDraft>,
        default_role: &str,
        target: Rgb,
        palette: &[Paint],
        locale: Locale,
    ) -> PlanLayer {
        let recipe = self
            .mix_service
            .suggest_mixes(target, palette, 1, None)
            .into_iter()
            .next();
        let achieved = recipe.as_ref().map(|r| r.color).unwrap_or(target);
        let delta_e = recipe.as_ref().map(|r| r.delta_e).unwrap_or(0.0);

        let role = draft
            .map(|d| d.role.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or(default_role)
            .to_string();
        let default_technique = match Self::system_of(palette) {
            PaintSystem::Oil => Technique::Blending,
            PaintSystem::Acrylic => Technique::Layering,
        }
        .label_i18n(locale);
        let technique = draft
            .map(|d| d.technique.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or(default_technique)
            .to_string();
        let note = draft.map(|d| d.note.clone()).unwrap_or_default();

        PlanLayer {
            role,
            target,
            technique,
            note,
            recipe,
            achieved,
            delta_e,
        }
    }

    fn parse_or(draft: Option<&LayerDraft>, fallback: Rgb) -> Rgb {
        draft
            .and_then(|d| Rgb::of_hex(&d.target_hex).ok())
            .unwrap_or(fallback)
    }

    fn midpoint(a: Rgb, b: Rgb) -> Rgb {
        Rgb::new(
            (a.r() + b.r()) / 2.0,
            (a.g() + b.g()) / 2.0,
            (a.b() + b.b()) / 2.0,
        )
    }
}

#[derive(Debug, Clone)]
pub struct SampledZone {
    pub name: String,
    pub material: String,
    pub target_hex: String,
}

impl SampledZone {
    pub fn new(name: impl Into<String>, material: impl Into<String>, target: Rgb) -> Self {
        Self {
            name: name.into(),
            material: material.into(),
            target_hex: target.to_hex(),
        }
    }
}

pub struct PaintingPlanService {
    engine: Arc<dyn ChatEngine>,
    decoder: ImageDecoder,
    enricher: PlanEnricher,
}

impl PaintingPlanService {
    const MAX_DIMENSION: u32 = 1024;

    const SYSTEM_PROMPT: &'static str = r##"Tu es un maître peintre de figurines d'exposition (bustes 1/10, 75mm, figurines 28-32mm, véhicules 1/35) qui travaille à l'huile sur sous-couche acrylique sèche, en appliquant rigoureusement les 14 règles d'atelier des guides de référence Norma Blue (guide-melanges-samourai-norma-blue.pdf et guide-melanges-norma-blue 3.pdf).

Pour chaque zone du sujet (ex. "Peau (chaude, ombres rosées-violacées)", "Cheveux (blond platine ou foncé)", "Veste / Haori (brun-olive)", "Katana / Armes (fourreau bleu et tsuba or NMM)", "Tissus & Kimono (blanc cassé)", "Petite créature (vert vif)", "Socle"...), construis une ÉCHELLE DE VALEURS complète à 5 marches, du plus sombre au plus clair :
  - shadow1    (role = "Ombre 2")   : ombre profonde, creux, occlusion maximale
  - shadow2    (role = "Ombre 1")   : ombre propre, demi-teinte d'ombre
  - base       (role = "Base")      : teinte locale à mi-valeur
  - highlight1 (role = "Lumière 1") : première montée de lumière
  - highlight2 (role = "Lumière 2") : éclat final sur les arêtes et crêtes exposées

Les 5 champs shadow1, shadow2, base, highlight1, highlight2 sont OBLIGATOIRES sur chaque zone. Leurs codes targetHex (#RRGGBB) doivent former une vraie progression de clarté : shadow1 nettement plus sombre que base, highlight2 nettement plus clair.

Règles chromatiques et techniques issues des guides d'atelier Norma Blue à respecter scrupuleusement :
1. Ombres vivantes (pas de noir pur) : Préfère toujours des ombres colorées (Bleu de Prusse + Garance ou Brun foncé) au noir minéral du tube. Le noir du tube salit et éteint les teintes ; il ne sert qu'aux olives ou au socle.
2. Peau humaine vivante : Les ombres rosées-violacées (Sienne + Garance + trace de Bleu de Prusse) donnent la vie au visage. Ajoute toujours une "Demi-teinte froide" (viridian/vert + blanc) sur la mâchoire et les tempes, et réserve les lumières les plus claires (Blanc + trace Jaune citron) au front, à l'arête du nez et aux pommettes.
3. Éclaircissement sans effet crayeux : Ne JAMAIS éclaircir les rouges, bruns et verts uniquement au blanc. Passe d'abord par le jaune (Jaune citron ou Ocre), et réserve le blanc pour le point spéculaire final.
4. Harmonisation générale : Une pointe d'Ocre jaune (le "ciment" de la palette) dans les mélanges lie et harmonise toutes les couleurs de la figurine.
5. Gestion des pigments puissants : Le Bleu de Prusse est ultra-puissant (prélever à la pointe d'un cure-dent). La Garance et le Viridian sont transparents (idéaux pour les glacis). Le Cadmium et l'Ocre sont opaques (meilleures bases).
6. Lavis d'huile en retrait : Sur sous-couche blanche ou zénithale bien sèche, le lavis se pose large puis se retire après 5 à 10 min au pinceau sec propre ou au coton-tige à peine humide de médium : le lavis reste dans les creux et dessine les volumes.
7. Ordre de peinture optimal : Lavis d'ombrage -> Peau -> Cheveux -> Vêtements / Kimono -> Cuirs & Armes -> Accessoires -> Socle. Peins une zone pendant qu'une autre sèche.
8. Texturation des tissus : Hachures croisées et griffures posées à la toute fin sur couche sèche au pinceau très fin (00 ou 000).
9. NMM (Métal non-métallique or ou acier) : Fort contraste entre ombre brune/noire profonde et éclat de lumière presque blanc pur.
10. Finition par vernis : Vernis mat général pour unifier les fondus, avec vernis satiné ponctuel sur les yeux, les lèvres et l'or.

Dans le champ "note" de chaque couche, indique PRÉCISÉMENT où poser la teinte sur la sculpture (ex. "joues, tempes, sous les pommettes", "orbites, sous le nez, mâchoire, cou", "front, arête du nez, pommettes, menton", "plis profonds, sous la veste", "arêtes du laçage").

Dans "accents", ajoute les sous-éléments et effets propres à la zone :
  - Pour un visage / peau : "Rougeurs" (joues, oreilles, bout du nez), "Demi-teinte froide" (tempes, mâchoire), "Barbe naissante", "Lèvres", "Yeux" (blanc, iris, pupille, reflet).
  - Pour cheveux, drapés, cuirs ou armes : "Reflet doré", "Reflet argenté", "Texture tissu (hachures croisées)", "Usure", "Revers / ombres violacées", "Garde / tsuba or".

Dans "generalApproach" et "warnings", formule ces conseils d'atelier Norma Blue concrets adaptés au sujet.

IMPORTANT : n'invente AUCUNE recette de mélange dans targetHex et ne cite aucun nom de tube : donne le code targetHex (#RRGGBB) exact de la teinte visée à chaque marche, la technique de pose et l'indication de placement dans "note". Le solveur Kubelka-Munk et le générateur de Guide de mélanges calculeront automatiquement les dosages exacts (B, JC, Rc, Ga, Pr, Vi, Oc, N, Si, BF, NC).

Réponds en français, uniquement avec le JSON demandé par le schéma."##;

    const SYSTEM_PROMPT_EN: &'static str = r##"You are a master display miniature painter (1/10 busts, 75mm figures, 28-32mm miniatures, 1/35 vehicles) working with oils over a dry acrylic undercoat, strictly applying the 14 studio rules from the Norma Blue reference guides (guide-melanges-samourai-norma-blue.pdf and guide-melanges-norma-blue 3.pdf).

For each zone of the subject (e.g. "Skin (warm, pinkish-violet shadows)", "Hair (platinum or dark blonde)", "Jacket / Haori (olive-brown)", "Katana / Weapons (blue scabbard, gold NMM tsuba)", "Fabrics & Kimono (off-white)", "Creature (vivid green)", "Base"...), build a complete 5-step VALUE SCALE from darkest to lightest:
  - shadow1    (role = "Shadow 2")    : deep shadow, recesses, maximum occlusion
  - shadow2    (role = "Shadow 1")    : core shadow, shadow halftone
  - base       (role = "Base")        : mid-value local tone
  - highlight1 (role = "Highlight 1") : first highlight step
  - highlight2 (role = "Highlight 2") : final specular highlight on exposed edges and ridges

All 5 fields shadow1, shadow2, base, highlight1, highlight2 are MANDATORY on every zone. Their targetHex (#RRGGBB) codes must form a true lightness progression: shadow1 noticeably darker than base, highlight2 noticeably lighter.

Chromatic and technical rules from the Norma Blue studio reference guides to strictly follow:
1. Vibrant shadows (no pure black): Always favor colored shadows (Prussian Blue + Madder Lake or Dark Brown) over tube mineral black. Tube black deadens colors; use it only for olive tones or bases.
2. Living human flesh: Warm pinkish-violet shadows (Sienna + Madder + trace of Prussian Blue) bring life to the face. Always include a "Cool halftone" (viridian + white) on jawline and temples, and reserve pure highlights (White + trace Lemon Yellow) for forehead, nose ridge, and cheekbones.
3. No chalky highlights: NEVER lighten reds, browns, or greens solely with white. Pass through yellow (Lemon Yellow or Yellow Ochre) first, saving pure white for the final specular glint.
4. Harmonious unification: A touch of Yellow Ochre (the "cement" of the palette) in mixes unifies and harmonizes all tones across the piece.
5. Pigment behavior: Prussian Blue is extremely strong (sample with a toothpick tip). Madder Lake and Viridian are transparent (ideal for glazes). Cadmium Red and Ochre are opaque (best basecoats).
6. Reductive oil wash: Over a fully dry white or zenithal undercoat, apply the wash generously then wick it back 5 to 10 min later with a clean dry brush or cotton swab lightly dampened with medium: wash stays in recesses and sculpts volumes.
7. Painting sequence: Wash undercoat -> Skin -> Hair -> Clothing / Kimono -> Leather & Weapons -> Accessories -> Base. Paint one area while another is drying.
8. Fabric texturing: Cross-hatching and fine scratches applied at the very end onto a touch-dry layer with a 00 or 000 brush.
9. NMM (Non-metallic metal): Extreme value contrast between deep dark brown/black shadows and nearly pure white specular catchlights.
10. Varnish finish: Overall dead matte varnish to unify blends, with spot satin varnish on eyes, lips, and gold metal.

In the "note" field of each layer, specify PRECISELY where to place the tone on the sculpt (e.g. "cheeks, temples, under cheekbones", "eye sockets, under nose, jawline, neck", "forehead, bridge of nose, cheekbones, chin", "deep folds, under jacket", "lacing edges").

In "accents", add sub-elements and effects specific to the zone:
  - For face / skin: "Blush" (cheeks, ears, nose tip), "Cool halftone" (temples, jawline), "Stubble", "Lips", "Eyes" (sclera, iris, pupil, catchlight).
  - For hair, drapery, leather, or weapons: "Golden reflection", "Silver reflection", "Fabric texture (cross-hatching)", "Weathering", "Lapels / violet shadows", "Gold guard / tsuba".

In "generalApproach" and "warnings", formulate these concrete Norma Blue studio tips tailored to the subject.

IMPORTANT: do NOT invent any mixing recipe in targetHex and do not cite any paint tube name: provide the exact targetHex (#RRGGBB) code of the target tone at each step, the application technique, and the placement note in "note". The Kubelka-Munk solver and the Mixing Guide generator will automatically compute the exact ratios.

Respond in English, only with the JSON requested by the schema."##;

    /// Same task, painted entirely in acrylic: no oil rule, no solvent, no drying in days.
    const ACRYLIC_SYSTEM_PROMPT: &'static str = r##"Tu es un maître peintre de figurines d'exposition (bustes 1/10, 75mm, figurines 28-32mm, véhicules 1/35) qui peint ENTIÈREMENT À L'ACRYLIQUE (peintures à l'eau pour figurines), sans huile ni solvant.

Pour chaque zone du sujet, construis une ÉCHELLE DE VALEURS complète à 5 marches, du plus sombre au plus clair :
  - shadow1    (role = "Ombre 2")   : ombre profonde, creux, occlusion maximale
  - shadow2    (role = "Ombre 1")   : ombre propre, demi-teinte d'ombre
  - base       (role = "Base")      : teinte locale à mi-valeur
  - highlight1 (role = "Lumière 1") : première montée de lumière
  - highlight2 (role = "Lumière 2") : éclat final sur les arêtes et crêtes exposées

Les 5 champs shadow1, shadow2, base, highlight1, highlight2 sont OBLIGATOIRES sur chaque zone. Leurs codes targetHex (#RRGGBB) doivent former une vraie progression de clarté.

Règles de l'acrylique à respecter :
1. L'acrylique sèche en quelques minutes : on ne fond pas dans le frais, on SUPERPOSE des couches fines (layering), chacune un peu plus claire et un peu plus petite que la précédente. Pour un fondu humide (wet blending), il faut un retardateur.
2. Consistance de lait : deux couches fines valent mieux qu'une épaisse, qui empâte les détails.
3. Les ombres se posent en lavis ou en glacis dilués au médium plutôt qu'à l'eau seule, pour éviter les auréoles ; pas de noir pur, des ombres colorées.
4. Éclaircir par le jaune ou l'ocre avant le blanc pour éviter l'effet crayeux ; le blanc pur est réservé au point de lumière final.
5. Liserés d'arêtes (edge highlight) au plat du pinceau sur les armures, armes et véhicules ; brossage à sec (drybrush) pour les textures (fourrure, socle, métal usé).
6. Ordre : sous-couche, aplats de base, lavis d'ombrage, reprise des couches, liserés, points de lumière, vernis.
7. Utilise pour "technique" l'un de ces libellés : "Aplat de base", "Lavis d'ombrage", "Glacis", "Jus capillaire (pin wash)", "Filtre", "Couches superposees (layering)", "Fondu humide (wet blending)", "Brossage a sec (drybrush)", "Liseres d'aretes (edge highlight)", "Eclaircis et points lumineux", "Peinture contraste (speedpaint)".

Dans le champ "note" de chaque couche, indique PRÉCISÉMENT où poser la teinte sur la sculpture. Dans "accents", ajoute les sous-éléments et effets propres à la zone (yeux, lèvres, rougeurs, usure, reflets...).

IMPORTANT : n'invente AUCUNE recette de mélange et ne cite aucun nom de pot : donne le code targetHex (#RRGGBB) exact de la teinte visée à chaque marche. Le solveur calculera les mélanges à partir des acryliques de la palette.

Réponds en français, uniquement avec le JSON demandé par le schéma."##;

    const ACRYLIC_SYSTEM_PROMPT_EN: &'static str = r##"You are a master display miniature painter (1/10 busts, 75mm figures, 28-32mm miniatures, 1/35 vehicles) painting ENTIRELY IN ACRYLICS (water-based miniature paints), with no oil and no solvent.

For each zone of the subject, build a complete 5-step VALUE LADDER, from darkest to lightest:
  - shadow1    (role = "Shadow 2")   : deep shadow, recesses, maximum occlusion
  - shadow2    (role = "Shadow 1")   : core shadow, shadow half-tone
  - base       (role = "Base")       : local colour at mid value
  - highlight1 (role = "Highlight 1"): first step of light
  - highlight2 (role = "Highlight 2"): final glint on exposed edges and ridges

The 5 fields shadow1, shadow2, base, highlight1, highlight2 are MANDATORY for every zone. Their targetHex codes (#RRGGBB) must form a real lightness progression.

Acrylic rules to follow:
1. Acrylic dries in minutes: do not blend wet-on-wet, STACK thin layers (layering), each slightly lighter and smaller than the one below. Wet blending requires a retarder.
2. Milk consistency: two thin coats beat one thick coat that clogs detail.
3. Shadows go on as washes or glazes thinned with medium rather than water alone, to avoid tide marks; no pure black, coloured shadows.
4. Lighten through yellow or ochre before white to avoid chalkiness; pure white is kept for the final glint.
5. Edge highlights with the flat of the brush on armour, weapons and vehicles; drybrush for textures (fur, bases, worn metal).
6. Order: undercoat, base coats, shade wash, layering back up, edge highlights, glints, varnish.
7. For "technique", use one of: "Base coat", "Shade wash", "Glaze", "Pin wash", "Filter", "Layering", "Wet blending", "Drybrush", "Edge highlight", "Highlights & catchlights", "Contrast paint (speedpaint)".

In each layer's "note", state PRECISELY where the tone goes on the sculpt. In "accents", add the sub-elements and effects of the zone (eyes, lips, flush, wear, reflections...).

IMPORTANT: do NOT invent any mixing recipe and do not cite any paint name: provide the exact targetHex (#RRGGBB) of the target tone at each step. The solver will compute mixes from the acrylics in the palette.

Respond in English, only with the JSON requested by the schema."##;

    pub fn new(engine: Arc<dyn ChatEngine>) -> Self {
        Self {
            engine,
            decoder: ImageDecoder,
            enricher: PlanEnricher::new(),
        }
    }

    pub async fn plan(
        &self,
        image_bytes: Option<&[u8]>,
        description: &str,
        palette_name: &str,
        palette: &[Paint],
        sampled_zones: &[SampledZone],
    ) -> DomainResult<PaintingPlan> {
        self.plan_i18n(
            image_bytes,
            description,
            palette_name,
            palette,
            sampled_zones,
            Locale::Fr,
        )
        .await
    }

    pub async fn plan_i18n(
        &self,
        image_bytes: Option<&[u8]>,
        description: &str,
        palette_name: &str,
        palette: &[Paint],
        sampled_zones: &[SampledZone],
        locale: Locale,
    ) -> DomainResult<PaintingPlan> {
        if image_bytes.is_none() && description.trim().is_empty() && sampled_zones.is_empty() {
            let msg = match locale {
                Locale::En => "Provide a photo, a description, or eyedropper-sampled zones.",
                Locale::Fr => {
                    "Donnez une photo, une description ou des zones relevées à la pipette."
                }
            };
            return Err(DomainError::InvalidArgument(msg.to_string()));
        }
        if palette.is_empty() {
            let msg = match locale {
                Locale::En => "No paint tubes available to build a plan.",
                Locale::Fr => "Aucun tube disponible pour établir un plan.",
            };
            return Err(DomainError::InvalidArgument(msg.to_string()));
        }

        let prompt = Self::build_prompt_i18n(description, palette, sampled_zones, locale);
        let mut attachments = Vec::new();
        if let Some(bytes) = image_bytes {
            let scaled = self.decoder.scale_to(bytes, Self::MAX_DIMENSION)?;
            attachments.push(ImageAttachment::jpeg(scaled));
        }

        let system_prompt = match (PlanEnricher::system_of(palette), locale) {
            (PaintSystem::Oil, Locale::En) => Self::SYSTEM_PROMPT_EN,
            (PaintSystem::Oil, Locale::Fr) => Self::SYSTEM_PROMPT,
            (PaintSystem::Acrylic, Locale::En) => Self::ACRYLIC_SYSTEM_PROMPT_EN,
            (PaintSystem::Acrylic, Locale::Fr) => Self::ACRYLIC_SYSTEM_PROMPT,
        };

        let raw = self
            .engine
            .complete(system_prompt, &prompt, &attachments, Some(&PLAN_SCHEMA))
            .await?;

        let draft = Self::parse_draft(&raw)?;
        Ok(self
            .enricher
            .enrich_i18n(&draft, palette_name, palette, locale))
    }

    pub fn parse_draft(raw: &str) -> DomainResult<PlanDraft> {
        let start = raw.find('{');
        let end = raw.rfind('}');
        let cleaned = match (start, end) {
            (Some(s), Some(e)) if e > s => &raw[s..=e],
            _ => raw,
        };
        serde_json::from_str::<PlanDraft>(cleaned).map_err(|e| {
            let preview: String = raw.chars().take(200).collect();
            DomainError::InvalidArgument(format!("Réponse du modèle illisible ({e}) : {preview}"))
        })
    }

    fn build_prompt_i18n(
        description: &str,
        palette: &[Paint],
        sampled_zones: &[SampledZone],
        locale: Locale,
    ) -> String {
        match locale {
            Locale::Fr => Self::build_prompt(description, palette, sampled_zones),
            Locale::En => {
                let mut out = String::from("Subject to paint: ");
                if description.trim().is_empty() {
                    out.push_str("(see attached photo)");
                } else {
                    out.push_str(description.trim());
                }
                out.push_str("\n\n");

                if !sampled_zones.is_empty() {
                    out.push_str(
                        "Eyedropper-sampled zones on the reference photo (keep these zones and use each sampled color as the \"base\" tone of its zone, then build the 2 darker shadows and 2 lighter highlights around it):\n",
                    );
                    for z in sampled_zones {
                        out.push_str(&format!(
                            "- {} ({}): base = {}\n",
                            z.name, z.material, z.target_hex
                        ));
                    }
                    out.push('\n');
                }

                let sample: Vec<String> = palette
                    .iter()
                    .take(40)
                    .map(|p| format!("{} ({})", p.display_name(), p.hex_color))
                    .collect();
                out.push_str(&format!(
                    "Available color range ({} tubes): {}",
                    palette.len(),
                    sample.join(", ")
                ));
                if palette.len() > 40 {
                    out.push_str(", ...");
                }
                out.push_str("\n\nPropose a complete zone-by-zone painting plan.");
                out
            }
        }
    }

    fn build_prompt(description: &str, palette: &[Paint], sampled_zones: &[SampledZone]) -> String {
        let mut out = String::from("Sujet à peindre : ");
        if description.trim().is_empty() {
            out.push_str("(voir la photo jointe)");
        } else {
            out.push_str(description.trim());
        }
        out.push_str("\n\n");

        if !sampled_zones.is_empty() {
            out.push_str(
                "Zones relevées à la pipette sur la photo de référence (conserve ces zones et utilise chaque couleur relevée comme teinte de \"base\" de sa zone, puis construis autour d'elle les 2 ombres plus sombres et les 2 lumières plus claires) :\n",
            );
            for z in sampled_zones {
                out.push_str(&format!(
                    "- {} ({}) : base = {}\n",
                    z.name, z.material, z.target_hex
                ));
            }
            out.push('\n');
        }

        let sample: Vec<String> = palette
            .iter()
            .take(40)
            .map(|p| format!("{} ({})", p.display_name(), p.hex_color))
            .collect();
        out.push_str(&format!(
            "Gamme chromatique disponible ({} tubes) : {}",
            palette.len(),
            sample.join(", ")
        ));
        if palette.len() > 40 {
            out.push_str(", ...");
        }
        out.push_str("\n\nPropose un plan de peinture complet par zones.");
        out
    }
}
