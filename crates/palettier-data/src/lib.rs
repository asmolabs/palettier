#![forbid(unsafe_code)]

pub mod backup;
pub mod db;
pub mod entity;
pub mod migration;
pub mod repository;
pub mod seeder;

pub use backup::{
    BackupArchive, BackupDocument, BackupExporter, BackupImporter, BackupLayer, BackupPaint,
    BackupPaintRef, BackupPalette, BackupPhoto, BackupProject, BackupRecipe, BackupStep,
    BackupWriter, BackupZone, ExportReport, ImportReport, ZipBackupArchive, ZipBackupWriter,
};
pub use db::Database;
pub use repository::{
    SeaOrmPaintCatalogRepository, SeaOrmPaletteMixRepository, SeaOrmPaletteRepository,
    SeaOrmProjectRepository, SeaOrmRecipeRepository, SeaOrmSettingsRepository,
};
pub use seeder::{CatalogSeeder, SeedReport};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration as ChronoDuration, Utc};
    use indexmap::IndexSet;
    use palettier_domain::color::Rgb;
    use palettier_domain::drying::Workshop;
    use palettier_domain::paint::{DryingClass, Opacity, Paint, Ventilation};
    use palettier_domain::palette::{Palette, PaletteMix, PaletteMixService};
    use palettier_domain::port::{
        PaintCatalogRepository, PaletteMixRepository, PaletteRepository, ProjectRepository,
    };
    use palettier_domain::project::{
        PhotoRole, Project, ProjectLayer, ProjectPhoto, ProjectZone,
    };
    use palettier_domain::workbench::WorkbenchService;
    use std::path::PathBuf;
    use std::sync::Arc;

    struct Installation {
        _db: Database,
        catalog: Arc<SeaOrmPaintCatalogRepository>,
        palettes: Arc<SeaOrmPaletteRepository>,
        projects: Arc<SeaOrmProjectRepository>,
        _recipes: Arc<SeaOrmRecipeRepository>,
        mixes: Arc<SeaOrmPaletteMixRepository>,
        seeder: CatalogSeeder,
        importer: BackupImporter,
        exporter: BackupExporter,
    }

    impl Installation {
        async fn in_memory() -> Self {
            let db = Database::in_memory().await.unwrap();
            Self::from_db(db)
        }

        fn from_db(db: Database) -> Self {
            let catalog = Arc::new(SeaOrmPaintCatalogRepository::new(&db));
            let palettes = Arc::new(SeaOrmPaletteRepository::new(&db, catalog.clone()));
            let projects = Arc::new(SeaOrmProjectRepository::new(
                &db,
                catalog.clone(),
                palettes.clone(),
            ));
            let recipes = Arc::new(SeaOrmRecipeRepository::new(&db));
            let mixes = Arc::new(SeaOrmPaletteMixRepository::new(&db));
            let seeder = CatalogSeeder::new(catalog.clone(), palettes.clone(), recipes.clone());
            let importer = BackupImporter::new(
                catalog.clone(),
                palettes.clone(),
                projects.clone(),
                recipes.clone(),
            );
            let exporter = BackupExporter::new(
                catalog.clone(),
                palettes.clone(),
                projects.clone(),
                recipes.clone(),
            );
            Self {
                _db: db,
                catalog,
                palettes,
                projects,
                _recipes: recipes,
                mixes,
                seeder,
                importer,
                exporter,
            }
        }
    }

    fn umber(in_stock: bool) -> Paint {
        Paint {
            id: None,
            brand: "Winsor & Newton".to_string(),
            name: "Burnt Umber".to_string(),
            code: "076".to_string(),
            legacy_code: String::new(),
            pigments: IndexSet::from(["PBr7".to_string()]),
            hex_color: "#4A3427".to_string(),
            tint_hex: Some("#C9B9AC".to_string()),
            opacity: Opacity::SemiOpaque,
            drying_class: DryingClass::Fast,
            tinting_strength: 0.8,
            color_derived: false,
            pigments_verified: true,
            user_added: false,
            in_stock,
            notes: String::new(),
        }
    }

    fn real_backup_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/sauvegarde-reelle.zip")
    }

    #[tokio::test]
    async fn saved_paint_reads_back_identically_with_pigments() {
        let inst = Installation::in_memory().await;
        let saved = inst.catalog.save(&umber(true)).await.unwrap();
        let id = saved.id.expect("id assigné");

        let read = inst.catalog.find_by_id(id).await.unwrap().expect("tube relu");
        assert_eq!("Burnt Umber", read.name);
        assert_eq!(IndexSet::from(["PBr7".to_string()]), read.pigments);
        assert_eq!(Some("#C9B9AC".to_string()), read.tint_hex);
        assert_eq!(DryingClass::Fast, read.drying_class);
        assert_eq!(Opacity::SemiOpaque, read.opacity);
        assert!(read.pigments_verified);
        assert!(read.in_stock);

        // The two-constant colorant uses a scattering distinct from 1.0
        let scattering = read.colorant().scattering();
        assert!(scattering.iter().any(|&s| (s - 1.0).abs() > 1e-6));
    }

    #[tokio::test]
    async fn stock_filter_and_calibration_updates_work() {
        let inst = Installation::in_memory().await;
        let u1 = inst.catalog.save(&umber(true)).await.unwrap();
        let mut raw = umber(false);
        raw.name = "Raw Umber".to_string();
        raw.tint_hex = None;
        raw.color_derived = true;
        let u2 = inst.catalog.save(&raw).await.unwrap();

        assert_eq!(2, inst.catalog.all().await.unwrap().len());
        assert_eq!(1, inst.catalog.count_owned().await.unwrap());
        assert_eq!(
            vec!["Burnt Umber".to_string()],
            inst.catalog
                .in_stock()
                .await
                .unwrap()
                .into_iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
        );

        inst.catalog.set_owned(u2.id.unwrap(), true).await.unwrap();
        assert_eq!(2, inst.catalog.count_owned().await.unwrap());

        inst.catalog
            .record_tint(u2.id.unwrap(), Rgb::of_hex("#D2C3B4").unwrap())
            .await
            .unwrap();
        inst.catalog
            .record_masstone(u2.id.unwrap(), Rgb::of_hex("#7E3A22").unwrap())
            .await
            .unwrap();

        let after = inst
            .catalog
            .find_by_id(u2.id.unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(Some("#D2C3B4".to_string()), after.tint_hex);
        assert_eq!("#7E3A22", after.hex_color);
        assert!(!after.color_derived);
        let _ = u1;
    }

    #[tokio::test]
    async fn project_saves_zones_layers_applied_coat_and_preserves_photos_on_resave() {
        let inst = Installation::in_memory().await;
        let paint = inst.catalog.save(&umber(true)).await.unwrap();
        let palette = inst
            .palettes
            .save(&Palette {
                id: None,
                name: "Zorn".to_string(),
                purpose: "Portraits".to_string(),
                notes: String::new(),
                paints: vec![paint.clone()],
            })
            .await
            .unwrap();

        let posed: DateTime<Utc> = "2026-03-01T18:30:00Z".parse().unwrap();
        let mut layer1 = ProjectLayer::new("Ombre 1", "#8A5F4A");
        layer1.technique = "Glacis".to_string();
        let layer1 = layer1.mark_applied(
            posed,
            Workshop::new(24.0, 40.0, Ventilation::Good).unwrap(),
            DryingClass::VerySlow,
        );
        let mut layer2 = ProjectLayer::new("Base", "#C98F72");
        layer2.technique = "Glacis".to_string();

        let zone = ProjectZone {
            id: None,
            name: "Visage".to_string(),
            material: "Peau".to_string(),
            note: String::new(),
            layers: vec![layer1, layer2],
        };
        let proj = Project {
            id: None,
            name: "Grognard".to_string(),
            subject: "Buste".to_string(),
            approach: "Par glacis.".to_string(),
            notes: String::new(),
            palette: Some(palette),
            paints: vec![paint],
            zones: vec![zone],
            photos: Vec::new(),
            created_at: posed,
        };

        let saved = inst.projects.save(&proj).await.unwrap();
        let proj_id = saved.id.unwrap();

        // Add a photo
        let with_photo = inst
            .projects
            .add_photo(
                proj_id,
                &ProjectPhoto {
                    id: None,
                    data: vec![1, 2, 3],
                    role: PhotoRole::Piece,
                    caption: "piece.png".to_string(),
                    added_at: posed,
                },
            )
            .await
            .unwrap();
        assert_eq!(1, with_photo.photos.len());

        // `all()` does not load photos, but re-saving this project must NOT delete the photo!
        let mut from_list = inst.projects.all().await.unwrap().into_iter().next().unwrap();
        assert!(from_list.photos.is_empty());
        from_list.notes = "une note de plus".to_string();
        inst.projects.save(&from_list).await.unwrap();

        let reread = inst
            .projects
            .find_with_photos(proj_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(1, reread.photos.len(), "les photos ont survécu à save()");
        assert_eq!("piece.png", reread.photos[0].caption);
        assert_eq!("une note de plus", reread.notes);

        let applied = reread.zones[0].layers[0].applied.as_ref().unwrap();
        assert_eq!(posed, applied.at);
        assert_eq!(24.0, applied.workshop.temperature_celsius);
        assert_eq!(Ventilation::Good, applied.workshop.ventilation);
        assert_eq!(DryingClass::VerySlow, applied.drying_class);

        // Two projects with the same name receive a unique suffix
        let mut dup = proj.clone();
        dup.id = None;
        let saved_dup = inst.projects.save(&dup).await.unwrap();
        assert_eq!("Grognard 2", saved_dup.name);
    }

    #[tokio::test]
    async fn palette_mix_persists_and_evaluates_open_time() {
        let inst = Installation::in_memory().await;
        let mixed_at: DateTime<Utc> = "2026-03-01T10:00:00Z".parse().unwrap();
        let mix = PaletteMix {
            id: None,
            name: "Gris rompu des ombres".to_string(),
            hex_color: "#6B6259".to_string(),
            recipe: "2 parts de terre d'ombre + 1 part de blanc".to_string(),
            drying_class: DryingClass::Medium,
            mixed_at,
            workshop: Workshop::new(24.0, 40.0, Ventilation::Good).unwrap(),
        };

        let saved = inst.mixes.save(&mix).await.unwrap();
        let read = inst.mixes.all().await.unwrap().into_iter().next().unwrap();
        assert_eq!("Gris rompu des ombres", read.name);
        assert_eq!(mixed_at, read.mixed_at);

        let service = PaletteMixService::default();
        assert!(service.state(&read, mixed_at).is_open());
        assert!(
            service
                .state(&read, mixed_at + ChronoDuration::days(20))
                .is_spent()
        );

        inst.mixes.delete(saved.id.unwrap()).await.unwrap();
        assert!(inst.mixes.all().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn desktop_database_reopens_cleanly_on_second_startup() {
        let temp_dir = std::env::temp_dir().join(format!(
            "palettier-rs-db-test-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        {
            let db1 = Database::open(&temp_dir).await.unwrap();
            let repo1 = SeaOrmPaintCatalogRepository::new(&db1);
            repo1.save(&umber(true)).await.unwrap();
            db1.close().await.unwrap();
        }
        {
            let db2 = Database::open(&temp_dir).await.unwrap();
            let repo2 = SeaOrmPaintCatalogRepository::new(&db2);
            let all = repo2.all().await.unwrap();
            assert_eq!(1, all.len());
            assert_eq!("Burnt Umber", all[0].name);
            assert_eq!(IndexSet::from(["PBr7".to_string()]), all[0].pigments);
            db2.close().await.unwrap();
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn catalog_seeder_populates_680_paints_and_is_idempotent() {
        let inst = Installation::in_memory().await;
        let report = inst.seeder.seed().await.unwrap();

        assert_eq!(680, report.paints_created);
        assert_eq!(3, report.palettes_added);
        assert_eq!(2, report.recipes_added);
        assert!(report.unknown_pigments.is_empty());

        let second = inst.seeder.seed().await.unwrap();
        assert!(!second.touched_anything(), "seeder non idempotent: {second:?}");

        let palettes = inst.palettes.all().await.unwrap();
        let zorn = palettes
            .iter()
            .find(|p| p.name.starts_with("Palette Zorn"))
            .unwrap();
        assert_eq!(4, zorn.paints.len());
        assert_eq!(DryingClass::VerySlow, zorn.slowest_drying_class());
    }

    #[tokio::test]
    async fn real_java_backup_imports_and_roundtrips_intact() {
        let inst1 = Installation::in_memory().await;
        let mut archive = ZipBackupArchive::open(real_backup_path()).unwrap();
        let report = inst1.importer.import(&mut archive).await.unwrap();

        assert_eq!(680, report.paints_created);
        assert_eq!(0, report.paints_updated);
        assert_eq!(1, report.projects_added);
        assert_eq!(1, report.photos);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);

        // Verify the recorded tint on Gamblin Burnt Umber
        let gamblin = inst1
            .catalog
            .find_by_natural_key("Gamblin", "Burnt Umber")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(Some("#C9B9AC".to_string()), gamblin.tint_hex);
        assert!(gamblin.in_stock);

        // Verify the workbench on the restored project
        let posed: DateTime<Utc> = "2026-02-14T18:30:00Z".parse().unwrap();
        let bench = WorkbenchService::default().bench(
            &inst1.projects.all().await.unwrap(),
            posed + ChronoDuration::days(30),
        );
        let zone = &bench.pieces[0].zones[0];
        assert!(zone.ready_now());
        assert_eq!(1, zone.applied);
        assert_eq!(6, zone.total);

        // Idempotent re-import
        let mut archive_again = ZipBackupArchive::open(real_backup_path()).unwrap();
        let second_import = inst1.importer.import(&mut archive_again).await.unwrap();
        assert_eq!(680, second_import.paints_updated);
        assert_eq!(0, second_import.paints_created);
        assert_eq!(0, second_import.projects_added);
        assert!(second_import.skipped() > 0);

        // Export to a new ZIP archive then re-import into a fresh installation
        let temp_zip = std::env::temp_dir().join(format!(
            "palettier-rs-roundtrip-{}.zip",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let mut writer = ZipBackupWriter::new(&temp_zip);
        let export_report = inst1.exporter.export(&mut writer).await.unwrap();
        writer.finish().unwrap();

        assert_eq!(680, export_report.paints);
        assert_eq!(1, export_report.projects);
        assert_eq!(1, export_report.photos);

        let inst2 = Installation::in_memory().await;
        let mut exported_archive = ZipBackupArchive::open(&temp_zip).unwrap();
        inst2.importer.import(&mut exported_archive).await.unwrap();

        assert_eq!(680, inst2.catalog.all().await.unwrap().len());
        let proj2 = inst2.projects.all().await.unwrap().into_iter().next().unwrap();
        assert_eq!("Grognard de reference", proj2.name);
        assert_eq!(6, proj2.zones[0].layers.len());
        let full_proj2 = inst2
            .projects
            .find_with_photos(proj2.id.unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(1, full_proj2.photos.len());
        assert_eq!("piece.png", full_proj2.photos[0].caption);

        let _ = std::fs::remove_file(&temp_zip);
    }
}
