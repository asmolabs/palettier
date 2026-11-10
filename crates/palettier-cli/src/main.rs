#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use palettier_data::{
    BackupExporter, BackupImporter, BackupWriter, CatalogSeeder, Database,
    SeaOrmPaintCatalogRepository, SeaOrmPaletteMixRepository, SeaOrmPaletteRepository,
    SeaOrmProjectRepository, SeaOrmRecipeRepository, ZipBackupArchive, ZipBackupWriter,
};
use palettier_domain::port::{
    PaintCatalogRepository, PaletteMixRepository, PaletteRepository, ProjectRepository,
    RecipeRepository,
};
use palettier_domain::workbench::WorkbenchService;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "palettier-cli",
    version,
    about = "Outil en ligne de commande Palettier (peinture à l'huile pour figurines)"
)]
struct Cli {
    /// Database directory or SQLite file path (defaults to `~/.palettier`)
    #[arg(long, global = true)]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes the database and synchronizes the catalog of 680 oils, palettes, and recipes
    Seed,
    /// Imports a backup archive (.zip) produced by Palettier
    ImportBackup {
        /// Path to the .zip file to import
        archive: PathBuf,
    },
    /// Exports the database contents to a backup archive (.zip)
    ExportBackup {
        /// Path to the .zip file to create
        output: PathBuf,
    },
    /// Displays the status of the catalog, projects, and workbench
    Stats,
}

fn default_db_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("PALETTIER_DB_DIR") {
        return PathBuf::from(dir);
    }
    let local = PathBuf::from(".").join(".palettier");
    if local.exists() {
        return local;
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".palettier")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(default_db_dir);
    let database = Database::open(&db_path)
        .await
        .with_context(|| format!("Impossible d'ouvrir la base dans {:?}", db_path))?;

    let catalog = Arc::new(SeaOrmPaintCatalogRepository::new(&database));
    let palettes = Arc::new(SeaOrmPaletteRepository::new(&database, catalog.clone()));
    let projects = Arc::new(SeaOrmProjectRepository::new(
        &database,
        catalog.clone(),
        palettes.clone(),
    ));
    let recipes = Arc::new(SeaOrmRecipeRepository::new(&database));
    let mixes = Arc::new(SeaOrmPaletteMixRepository::new(&database));

    match cli.command {
        Commands::Seed => {
            let seeder = CatalogSeeder::new(catalog, palettes, recipes);
            let report = seeder.seed().await?;
            println!(
                "Amorcage terminé ({:?}) : {} tubes créés, {} rafraîchis, {} retirés, {} teintes diluées complétées, {} palettes ajoutées, {} recettes ajoutées.",
                db_path,
                report.paints_created,
                report.paints_refreshed,
                report.paints_removed,
                report.tints_backfilled,
                report.palettes_added,
                report.recipes_added
            );
        }
        Commands::ImportBackup { archive } => {
            let importer = BackupImporter::new(catalog, palettes, projects, recipes);
            let mut zip = ZipBackupArchive::open(&archive)?;
            let report = importer.import(&mut zip).await?;
            println!(
                "Restauration depuis {:?} vers {:?} : {} tubes ajoutés, {} mis à jour, {} palettes, {} projets, {} recettes, {} photos ({} ignorés).",
                archive,
                db_path,
                report.paints_created,
                report.paints_updated,
                report.palettes_added,
                report.projects_added,
                report.recipes_added,
                report.photos,
                report.skipped()
            );
            for w in &report.warnings {
                println!("  avertissement : {w}");
            }
        }
        Commands::ExportBackup { output } => {
            let exporter = BackupExporter::new(catalog, palettes, projects, recipes);
            let mut writer = ZipBackupWriter::new(&output);
            let report = exporter.export(&mut writer).await?;
            writer.finish()?;
            println!(
                "Sauvegarde écrite dans {:?} : {} tubes, {} palettes, {} projets, {} recettes, {} photos.",
                output, report.paints, report.palettes, report.projects, report.recipes, report.photos
            );
        }
        Commands::Stats => {
            let total = catalog.count().await?;
            let owned = catalog.count_owned().await?;
            let all_palettes = palettes.all().await?;
            let all_projects = projects.all().await?;
            let all_recipes = recipes.all().await?;
            let all_mixes = mixes.all().await?;
            let bench = WorkbenchService::default().bench(&all_projects, Utc::now());

            println!("Base Palettier : {:?}", db_path);
            println!("  Catalogue    : {total} tubes ({owned} en stock)");
            println!("  Palettes     : {}", all_palettes.len());
            println!("  Projets      : {}", all_projects.len());
            println!("  Recettes     : {}", all_recipes.len());
            println!("  Mélanges     : {}", all_mixes.len());
            println!(
                "  Établi       : {} pièces prêtes, {} en attente",
                bench.ready().len(),
                bench.waiting().len()
            );
        }
    }

    database.close().await?;
    Ok(())
}
