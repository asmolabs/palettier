use indexmap::{IndexMap, IndexSet};
use palettier_domain::catalog::CatalogLoader;
use palettier_domain::color::Rgb;
use palettier_domain::error::DomainResult;
use palettier_domain::paint::{LayerThickness, Medium, Paint, Technique};
use palettier_domain::palette::Palette;
use palettier_domain::port::{PaintCatalogRepository, PaletteRepository, RecipeRepository};
use palettier_domain::recipe::{Recipe, RecipeStep};
use std::sync::Arc;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeedReport {
    pub paints_created: usize,
    pub paints_refreshed: usize,
    pub paints_removed: usize,
    pub paints_kept_in_use: usize,
    pub paints_kept_calibrated: Vec<String>,
    pub tints_backfilled: usize,
    pub palettes_added: usize,
    pub recipes_added: usize,
    pub unknown_pigments: Vec<String>,
}

impl SeedReport {
    pub fn touched_anything(&self) -> bool {
        self.paints_created > 0
            || self.paints_refreshed > 0
            || self.paints_removed > 0
            || self.tints_backfilled > 0
            || self.palettes_added > 0
            || self.recipes_added > 0
    }
}

#[derive(Default)]
struct SyncStats {
    refreshed: usize,
    added: usize,
    removed: usize,
    kept_in_use: usize,
    kept_calibrated: Vec<String>,
}

pub struct CatalogSeeder {
    paints: Arc<dyn PaintCatalogRepository>,
    palettes: Arc<dyn PaletteRepository>,
    recipes: Arc<dyn RecipeRepository>,
    loader: CatalogLoader,
}

impl CatalogSeeder {
    pub fn new(
        paints: Arc<dyn PaintCatalogRepository>,
        palettes: Arc<dyn PaletteRepository>,
        recipes: Arc<dyn RecipeRepository>,
    ) -> Self {
        Self {
            paints,
            palettes,
            recipes,
            loader: CatalogLoader::default(),
        }
    }

    pub fn with_loader(
        paints: Arc<dyn PaintCatalogRepository>,
        palettes: Arc<dyn PaletteRepository>,
        recipes: Arc<dyn RecipeRepository>,
        loader: CatalogLoader,
    ) -> Self {
        Self {
            paints,
            palettes,
            recipes,
            loader,
        }
    }

    pub async fn seed(&self) -> DomainResult<SeedReport> {
        let reference = self.loader.load_detailed();

        let created = if self.paints.count().await? == 0 {
            self.paints.save_all(&reference.paints).await?;
            reference.paints.len()
        } else {
            0
        };

        let sync = if created == 0 {
            self.sync_catalogue(&reference.paints).await?
        } else {
            SyncStats::default()
        };

        let tints = self.backfill_tints(&reference.paints).await?;
        let palettes_added = self.seed_missing_palettes().await?;
        let recipes_added = self.seed_missing_recipes().await?;

        Ok(SeedReport {
            paints_created: created + sync.added,
            paints_refreshed: sync.refreshed,
            paints_removed: sync.removed,
            paints_kept_in_use: sync.kept_in_use,
            paints_kept_calibrated: sync.kept_calibrated,
            tints_backfilled: tints,
            palettes_added,
            recipes_added,
            unknown_pigments: reference.unknown_pigments,
        })
    }

    async fn sync_catalogue(&self, reference: &[Paint]) -> DomainResult<SyncStats> {
        let mut sync = SyncStats::default();
        let from_files: IndexMap<String, &Paint> =
            reference.iter().map(|p| (Self::key(p), p)).collect();
        let covered_brands: IndexSet<String> = reference.iter().map(|p| p.brand.clone()).collect();

        let stored = self.paints.all().await?;
        let mut obsolete = Vec::new();

        for paint in &stored {
            let k = Self::key(paint);
            if let Some(&file_paint) = from_files.get(&k) {
                let refreshed = Self::refresh_identity(paint, file_paint);
                if &refreshed != paint {
                    self.paints.refresh_identity(&refreshed).await?;
                    sync.refreshed += 1;
                }
            } else if !paint.user_added && covered_brands.contains(&paint.brand) {
                obsolete.push(paint.clone());
            }
        }

        let known: IndexSet<String> = stored.iter().map(Self::key).collect();
        for (file_key, &paint) in &from_files {
            if !known.contains(file_key) {
                self.paints.save(paint).await?;
                sync.added += 1;
            }
        }

        self.remove_obsolete(&obsolete, &mut sync).await?;
        Ok(sync)
    }

    fn refresh_identity(stored: &Paint, reference: &Paint) -> Paint {
        let mut updated = stored.clone();
        updated.code = reference.code.clone();
        updated.legacy_code = reference.legacy_code.clone();
        updated.pigments = reference.pigments.clone();
        updated.opacity = reference.opacity;
        updated.drying_class = reference.drying_class;
        updated.tinting_strength = reference.tinting_strength;
        updated.pigments_verified = reference.pigments_verified;
        updated.system = reference.system;

        if updated.color_derived && !reference.color_derived {
            updated.hex_color = reference.hex_color.clone();
            updated.color_derived = false;
        }
        updated
    }

    async fn remove_obsolete(&self, obsolete: &[Paint], sync: &mut SyncStats) -> DomainResult<()> {
        if obsolete.is_empty() {
            return Ok(());
        }
        let in_use = self.paints.ids_in_use().await?;

        for paint in obsolete {
            let Some(id) = paint.id else { continue };
            if in_use.contains(&id) {
                sync.kept_in_use += 1;
            } else if Self::is_calibrated(paint) {
                sync.kept_calibrated.push(paint.display_name());
            } else {
                self.paints.delete(id).await?;
                sync.removed += 1;
            }
        }
        Ok(())
    }

    fn is_calibrated(paint: &Paint) -> bool {
        paint
            .tint_hex
            .as_ref()
            .is_some_and(|s| !s.trim().is_empty())
            || !paint.notes.trim().is_empty()
    }

    async fn backfill_tints(&self, reference: &[Paint]) -> DomainResult<usize> {
        let from_files: IndexMap<String, String> = reference
            .iter()
            .filter_map(|p| p.tint_hex.as_ref().map(|t| (Self::key(p), t.clone())))
            .collect();

        let mut updated = 0;
        for paint in self.paints.all().await? {
            if paint.tint_hex.is_some() {
                continue;
            }
            let Some(tint_str) = from_files.get(&Self::key(&paint)) else {
                continue;
            };
            if let (Some(id), Ok(rgb)) = (paint.id, Rgb::of_hex(tint_str)) {
                self.paints.record_tint(id, rgb).await?;
                updated += 1;
            }
        }
        Ok(updated)
    }

    fn key(paint: &Paint) -> String {
        format!("{}|{}", paint.brand, paint.name)
    }

    async fn seed_missing_palettes(&self) -> DomainResult<usize> {
        let existing: IndexSet<String> = self
            .palettes
            .all()
            .await?
            .into_iter()
            .map(|p| p.name.to_lowercase())
            .collect();

        let mut added = 0;
        for candidate in self.reference_palettes().await? {
            if existing.contains(&candidate.name.to_lowercase()) {
                continue;
            }
            self.palettes.save(&candidate).await?;
            added += 1;
        }
        Ok(added)
    }

    async fn seed_missing_recipes(&self) -> DomainResult<usize> {
        let mut added = 0;
        for candidate in Self::default_recipes() {
            if self.recipes.exists(&candidate.name).await? {
                continue;
            }
            self.recipes.save(&candidate).await?;
            added += 1;
        }
        Ok(added)
    }

    async fn reference_palettes(&self) -> DomainResult<Vec<Palette>> {
        Ok(vec![
            self.zorn().await?,
            self.flesh().await?,
            self.armour().await?,
            self.acrylic_armour().await?,
        ])
    }

    async fn acrylic_armour(&self) -> DomainResult<Palette> {
        Ok(Palette {
            id: None,
            name: "Armure bleue (acrylique)".to_string(),
            purpose: "Figurine 28-32 mm, peinte entierement a l'acrylique".to_string(),
            notes: concat!(
                "Une echelle de bleus du plus sombre au plus clair, un lavis noir-bleu pour les creux ",
                "et un gris tres clair pour les liseres. A l'acrylique, on ne fond pas : on superpose ",
                "des couches fines, chacune un peu plus claire et un peu plus petite que la precedente."
            )
            .to_string(),
            paints: self
                .resolve(&[
                    ("Citadel", "Kantor Blue"),
                    ("Citadel", "Macragge Blue"),
                    ("Citadel", "Altdorf Guard Blue"),
                    ("Citadel", "Calgar Blue"),
                    ("Citadel", "Fenrisian Grey"),
                    ("Citadel", "Abaddon Black"),
                    ("Citadel", "White Scar"),
                ])
                .await?,
        })
    }

    async fn zorn(&self) -> DomainResult<Palette> {
        Ok(Palette {
            id: None,
            name: "Palette Zorn (Winsor & Newton)".to_string(),
            purpose: "Carnations et portraits".to_string(),
            notes: concat!(
                "Quatre tubes : blanc de titane, ocre jaune, cadmium ecarlate, noir d'ivoire.\n\n",
                "Le noir ne sert pas a noircir, il tient le role du bleu. Sorti du tube il est chaud ; coupe de blanc il donne des gris franchement bleutes, et mele a l'ocre des verts sourds. Les ombres d'une carnation se font donc au noir, sans jamais ajouter de bleu.\n\n",
                "Cette divergence entre le tube et le melange est reelle, et c'est ce que le modele a deux constantes de l'application sait reproduire.\n\n",
                "Zorn peignait au vermillon ; le cadmium ecarlate le remplace. C'est le pigment le plus lent de la palette, il commande le planning de la seance."
            )
            .to_string(),
            paints: self
                .resolve(&[
                    ("Winsor & Newton", "Titanium White"),
                    ("Winsor & Newton", "Yellow Ochre"),
                    ("Winsor & Newton", "Cadmium Scarlet"),
                    ("Winsor & Newton", "Ivory Black"),
                ])
                .await?,
        })
    }

    async fn flesh(&self) -> DomainResult<Palette> {
        Ok(Palette {
            id: None,
            name: "Carnations 1/10".to_string(),
            purpose: "Visages et mains de buste".to_string(),
            notes: concat!(
                "Le blanc et le rouge portent les lumieres, la terre d'ombre et le bleu de ",
                "Prusse portent les ombres. Tout le reste se trouve en melangeant ces deux ",
                "extremites : une palette courte donne des carnations plus coherentes qu'une ",
                "palette fournie."
            )
            .to_string(),
            paints: self
                .resolve(&[
                    ("Abteilung 502", "Basic Flesh Tone"),
                    ("Abteilung 502", "Flesh Shadow"),
                    ("Abteilung 502", "Light Flesh Tone"),
                    ("Winsor & Newton", "Naples Yellow"),
                    ("Winsor & Newton", "Alizarin Crimson"),
                    ("Winsor & Newton", "Burnt Umber"),
                    ("Winsor & Newton", "Prussian Blue"),
                    ("Winsor & Newton", "Titanium White"),
                ])
                .await?,
        })
    }

    async fn armour(&self) -> DomainResult<Palette> {
        Ok(Palette {
            id: None,
            name: "Blindage vert olive".to_string(),
            purpose: "Vehicule 1/35".to_string(),
            notes: "Filtres, jus et salissures. Le blanc ne sert qu'a casser le vert, jamais a eclaircir.".to_string(),
            paints: self
                .resolve(&[
                    ("Winsor & Newton", "Olive Green"),
                    ("Winsor & Newton", "Yellow Ochre"),
                    ("Winsor & Newton", "Ivory Black"),
                    ("Abteilung 502", "Cassel Earth/Shadow Brown"),
                    ("Abteilung 502", "Starship Filth"),
                    ("Abteilung 502", "Red Ochre/Light Rust"),
                    ("Gamblin", "Titanium White"),
                ])
                .await?,
        })
    }

    async fn resolve(&self, keys: &[(&str, &str)]) -> DomainResult<Vec<Paint>> {
        let mut out = Vec::new();
        for &(brand, name) in keys {
            if let Some(p) = self.paints.find_by_natural_key(brand, name).await? {
                out.push(p);
            }
        }
        Ok(out)
    }

    fn default_recipes() -> Vec<Recipe> {
        vec![
            Recipe {
                id: None,
                name: "Carnation 1/10 a l'huile".to_string(),
                subject: "Visage de buste".to_string(),
                notes: concat!(
                    "Base acrylique poncee et vernie brillante, seche depuis 24 h.\n",
                    "Tout le modele repose sur des couches tres fines : on construit la profondeur\n",
                    "par accumulation, jamais en une seule passe chargee."
                )
                .to_string(),
                steps: vec![
                    Self::step(
                        Technique::BaseLayer,
                        "Basic Flesh Tone + une pointe de Naples Yellow",
                        Medium::OdorlessThinner,
                        0.25,
                        LayerThickness::Thin,
                        "Aplat tres fin, on doit encore deviner la base acrylique.",
                    ),
                    Self::step(
                        Technique::DotFading,
                        "Points de Deep Shadow Flesh, Naples Yellow et Alizarin Crimson",
                        Medium::None,
                        0.0,
                        LayerThickness::Thin,
                        "Rouge sur les pommettes, le nez et les oreilles ; jaune sur le front.",
                    ),
                    Self::step(
                        Technique::Blending,
                        "Fondu au pinceau propre",
                        Medium::None,
                        0.0,
                        LayerThickness::Thin,
                        "Tirer les transitions tant que la couche est ouverte, essuyer le pinceau a chaque passage.",
                    ),
                    Self::step(
                        Technique::PinWash,
                        "Burnt Umber + une pointe de Prussian Blue",
                        Medium::OdorlessThinner,
                        0.90,
                        LayerThickness::Glaze,
                        "Creux des paupieres, ailes du nez, commissures.",
                    ),
                    Self::step(
                        Technique::Highlight,
                        "Snow White + Basic Flesh Tone",
                        Medium::None,
                        0.0,
                        LayerThickness::Thin,
                        "Arete du nez, arcade, levre inferieure. Trois points suffisent.",
                    ),
                ],
            },
            Recipe {
                id: None,
                name: "Usure d'un blindage vert olive".to_string(),
                subject: "Vehicule 1/35".to_string(),
                notes: "Enchainement classique : filtre, jus, coulures, puis poussieres.".to_string(),
                steps: vec![
                    Self::step(
                        Technique::Filter,
                        "Olive Green tres dilue",
                        Medium::OdorlessThinner,
                        0.94,
                        LayerThickness::Glaze,
                        "Unifie les modules de camouflage sans les effacer.",
                    ),
                    Self::step(
                        Technique::PinWash,
                        "Cassel Earth/Shadow Brown",
                        Medium::OdorlessThinner,
                        0.90,
                        LayerThickness::Glaze,
                        "Uniquement dans les lignes de structure et autour des rivets.",
                    ),
                    Self::step(
                        Technique::StreakingGrime,
                        "Starship Filth",
                        Medium::OdorlessThinner,
                        0.75,
                        LayerThickness::Thin,
                        "Traits verticaux depuis les asperites, tires vers le bas apres deux minutes.",
                    ),
                    Self::step(
                        Technique::Glaze,
                        "Rust dans les zones d'ecaillage",
                        Medium::OdorlessThinner,
                        0.85,
                        LayerThickness::Glaze,
                        "Toujours au-dessus de l'ecaillage, jamais en dessous.",
                    ),
                ],
            },
            Recipe {
                id: None,
                name: "Armure bleue en couches acryliques".to_string(),
                subject: "Figurine 28-32 mm".to_string(),
                notes: concat!(
                    "Entierement a l'acrylique, sur sous-couche noire ou grise.
",
                    "Chaque etape se reprend apres quelques minutes : la piece se termine en une seance."
                )
                .to_string(),
                steps: vec![
                    Self::step(
                        Technique::BaseLayer,
                        "Macragge Blue",
                        Medium::Water,
                        0.30,
                        LayerThickness::Normal,
                        "Deux couches fines plutot qu'une epaisse.",
                    ),
                    Self::step(
                        Technique::ShadeWash,
                        "Kantor Blue + Abaddon Black (1:1)",
                        Medium::AcrylicGlazeMedium,
                        0.60,
                        LayerThickness::Glaze,
                        "Dans les creux et sous les plaques, sans repasser.",
                    ),
                    Self::step(
                        Technique::Layering,
                        "Macragge Blue puis Altdorf Guard Blue",
                        Medium::Water,
                        0.40,
                        LayerThickness::Thin,
                        "Reprendre la base en laissant l'ombre dans les creux, puis remonter sur les volumes.",
                    ),
                    Self::step(
                        Technique::EdgeHighlight,
                        "Calgar Blue",
                        Medium::Water,
                        0.30,
                        LayerThickness::Thin,
                        "Sur toutes les aretes, au plat du pinceau.",
                    ),
                    Self::step(
                        Technique::Highlight,
                        "Fenrisian Grey",
                        Medium::None,
                        0.0,
                        LayerThickness::Thin,
                        "Points de lumiere aux angles les plus exposes seulement.",
                    ),
                ],
            },
        ]
    }

    fn step(
        technique: Technique,
        mix: &str,
        medium: Medium,
        ratio: f64,
        thickness: LayerThickness,
        notes: &str,
    ) -> RecipeStep {
        RecipeStep {
            id: None,
            technique,
            paint_mix: mix.to_string(),
            medium,
            medium_ratio: ratio,
            thickness,
            notes: notes.to_string(),
        }
    }
}
