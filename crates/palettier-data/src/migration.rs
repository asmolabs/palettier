use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(InitialSchemaMigration),
            Box::new(PaintSystemMigration),
            Box::new(LayerTechniqueCodeMigration),
            Box::new(OrphanLayerCleanupMigration),
        ]
    }
}

#[derive(DeriveMigrationName)]
pub struct InitialSchemaMigration;

#[async_trait::async_trait]
impl MigrationTrait for InitialSchemaMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS paint (
                id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                brand              TEXT    NOT NULL,
                name               TEXT    NOT NULL,
                code               TEXT    NOT NULL DEFAULT '',
                legacy_code        TEXT    NOT NULL DEFAULT '',
                hex_color          TEXT    NOT NULL,
                tint_hex           TEXT,
                opacity            TEXT    NOT NULL,
                drying_class       TEXT    NOT NULL,
                tinting_strength   REAL    NOT NULL,
                colour_derived     INTEGER NOT NULL DEFAULT 0,
                pigments_verified  INTEGER NOT NULL DEFAULT 0,
                user_added         INTEGER NOT NULL DEFAULT 0,
                in_stock           INTEGER NOT NULL DEFAULT 1,
                notes              TEXT    NOT NULL DEFAULT ''
            );

            CREATE UNIQUE INDEX IF NOT EXISTS paint_natural_key ON paint(brand, name);
            CREATE INDEX IF NOT EXISTS paint_in_stock ON paint(in_stock);

            CREATE TABLE IF NOT EXISTS paint_pigment (
                paint_id INTEGER NOT NULL REFERENCES paint(id) ON DELETE CASCADE,
                pigment  TEXT    NOT NULL,
                position INTEGER NOT NULL,
                PRIMARY KEY (paint_id, pigment)
            );

            CREATE TABLE IF NOT EXISTS palette (
                id      INTEGER PRIMARY KEY AUTOINCREMENT,
                name    TEXT NOT NULL,
                purpose TEXT NOT NULL DEFAULT '',
                notes   TEXT NOT NULL DEFAULT ''
            );

            CREATE UNIQUE INDEX IF NOT EXISTS palette_name ON palette(name);

            CREATE TABLE IF NOT EXISTS palette_paint (
                palette_id INTEGER NOT NULL REFERENCES palette(id) ON DELETE CASCADE,
                paint_id   INTEGER NOT NULL REFERENCES paint(id) ON DELETE CASCADE,
                position   INTEGER NOT NULL,
                PRIMARY KEY (palette_id, paint_id)
            );

            CREATE TABLE IF NOT EXISTS project (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT    NOT NULL,
                subject    TEXT    NOT NULL DEFAULT '',
                approach   TEXT    NOT NULL DEFAULT '',
                notes      TEXT    NOT NULL DEFAULT '',
                palette_id INTEGER REFERENCES palette(id) ON DELETE SET NULL,
                created_at INTEGER NOT NULL
            );

            CREATE UNIQUE INDEX IF NOT EXISTS project_name ON project(name);

            CREATE TABLE IF NOT EXISTS project_paint (
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                paint_id   INTEGER NOT NULL REFERENCES paint(id) ON DELETE CASCADE,
                position   INTEGER NOT NULL,
                PRIMARY KEY (project_id, paint_id)
            );

            CREATE TABLE IF NOT EXISTS project_zone (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                name       TEXT    NOT NULL,
                material   TEXT    NOT NULL DEFAULT '',
                note       TEXT    NOT NULL DEFAULT '',
                position   INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS project_layer (
                id                   INTEGER PRIMARY KEY AUTOINCREMENT,
                zone_id              INTEGER NOT NULL REFERENCES project_zone(id) ON DELETE CASCADE,
                role                 TEXT    NOT NULL,
                target_hex           TEXT    NOT NULL,
                technique            TEXT    NOT NULL DEFAULT '',
                note                 TEXT    NOT NULL DEFAULT '',
                kind                 TEXT    NOT NULL DEFAULT 'LADDER',
                position             INTEGER NOT NULL,
                applied_at           INTEGER,
                applied_temperature  REAL,
                applied_humidity     REAL,
                applied_ventilation  TEXT,
                applied_drying_class TEXT
            );

            CREATE TABLE IF NOT EXISTS project_photo (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
                data       BLOB    NOT NULL,
                role       TEXT    NOT NULL,
                caption    TEXT    NOT NULL DEFAULT '',
                added_at   INTEGER NOT NULL,
                position   INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS recipe (
                id      INTEGER PRIMARY KEY AUTOINCREMENT,
                name    TEXT NOT NULL,
                subject TEXT NOT NULL DEFAULT '',
                notes   TEXT NOT NULL DEFAULT ''
            );

            CREATE UNIQUE INDEX IF NOT EXISTS recipe_name ON recipe(name);

            CREATE TABLE IF NOT EXISTS recipe_step (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                recipe_id    INTEGER NOT NULL REFERENCES recipe(id) ON DELETE CASCADE,
                technique    TEXT    NOT NULL,
                paint_mix    TEXT    NOT NULL DEFAULT '',
                medium       TEXT    NOT NULL,
                medium_ratio REAL    NOT NULL,
                thickness    TEXT    NOT NULL,
                notes        TEXT    NOT NULL DEFAULT '',
                position     INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS palette_mix (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                name         TEXT    NOT NULL,
                hex_color    TEXT    NOT NULL,
                recipe       TEXT    NOT NULL DEFAULT '',
                drying_class TEXT    NOT NULL,
                mixed_at     INTEGER NOT NULL,
                temperature  REAL    NOT NULL,
                humidity     REAL    NOT NULL,
                ventilation  TEXT    NOT NULL
            );

            CREATE TABLE IF NOT EXISTS setting (
                key   TEXT NOT NULL PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            DROP TABLE IF EXISTS setting;
            DROP TABLE IF EXISTS palette_mix;
            DROP TABLE IF EXISTS recipe_step;
            DROP TABLE IF EXISTS recipe;
            DROP TABLE IF EXISTS project_photo;
            DROP TABLE IF EXISTS project_layer;
            DROP TABLE IF EXISTS project_zone;
            DROP TABLE IF EXISTS project_paint;
            DROP TABLE IF EXISTS project;
            DROP TABLE IF EXISTS palette_paint;
            DROP TABLE IF EXISTS palette;
            DROP TABLE IF EXISTS paint_pigment;
            DROP TABLE IF EXISTS paint;
            "#,
        )
        .await?;
        Ok(())
    }
}

/// Oil or acrylic, on tubes, projects and palette mixes. Existing rows were all oil.
pub struct PaintSystemMigration;

// Both migrations live in this file: the derived name (the file name) would collide.
impl MigrationName for PaintSystemMigration {
    fn name(&self) -> &str {
        "m0002_paint_system"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for PaintSystemMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            ALTER TABLE paint       ADD COLUMN paint_system TEXT NOT NULL DEFAULT 'OIL';
            ALTER TABLE project     ADD COLUMN paint_system TEXT NOT NULL DEFAULT 'OIL';
            ALTER TABLE palette_mix ADD COLUMN paint_system TEXT NOT NULL DEFAULT 'OIL';
            CREATE INDEX IF NOT EXISTS paint_system ON paint(paint_system);
            "#,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            DROP INDEX IF EXISTS paint_system;
            ALTER TABLE palette_mix DROP COLUMN paint_system;
            ALTER TABLE project     DROP COLUMN paint_system;
            ALTER TABLE paint       DROP COLUMN paint_system;
            "#,
        )
        .await?;
        Ok(())
    }
}

/// A coat's technique was stored as its displayed label (French, English, with or
/// without accents, or anything the assistant made up). It is now stored as its code.
/// A label that names no known technique becomes the default technique of the piece's
/// system, and the label is kept, word for word, at the head of the coat's note.
///
/// Every row is converted, including coats left behind by a zone deleted while foreign
/// keys were off (they are read as oil): no row keeps a label the new code would misread.
pub struct LayerTechniqueCodeMigration;

impl MigrationName for LayerTechniqueCodeMigration {
    fn name(&self) -> &str {
        "m0003_layer_technique_code"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for LayerTechniqueCodeMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use palettier_domain::paint::{PaintSystem, Technique};
        use palettier_domain::project::ProjectLayer;
        use sea_orm::{ConnectionTrait, Statement};
        use std::str::FromStr;

        let db = manager.get_connection();
        let backend = db.get_database_backend();
        let rows = db
            .query_all_raw(Statement::from_string(
                backend,
                r#"
                SELECT l.id, l.technique, l.note, COALESCE(p.paint_system, 'OIL') AS paint_system
                FROM project_layer l
                LEFT JOIN project_zone z ON z.id = l.zone_id
                LEFT JOIN project p ON p.id = z.project_id
                "#,
            ))
            .await?;

        for row in rows {
            let id: i64 = row.try_get("", "id")?;
            let label: String = row.try_get("", "technique")?;
            let note: String = row.try_get("", "note")?;
            let system: String = row.try_get("", "paint_system")?;
            let system = PaintSystem::from_str(&system).unwrap_or_default();

            let mut layer = ProjectLayer::new("", "");
            layer.note = note;
            if label.trim().is_empty() {
                layer.technique = Technique::default_for(system);
            } else {
                layer.set_technique_label(&label, system);
            }
            db.execute_raw(Statement::from_sql_and_values(
                backend,
                "UPDATE project_layer SET technique = ?, note = ? WHERE id = ?",
                [
                    layer.technique.as_db_str().into(),
                    layer.note.into(),
                    id.into(),
                ],
            ))
            .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Codes back to French labels, the form the previous version reads. The labels
        // kept in notes stay there.
        use palettier_domain::paint::Technique;
        use sea_orm::{ConnectionTrait, Statement};

        let db = manager.get_connection();
        let backend = db.get_database_backend();
        for t in Technique::ALL {
            db.execute_raw(Statement::from_sql_and_values(
                backend,
                "UPDATE project_layer SET technique = ? WHERE technique = ?",
                [t.legacy_label().into(), t.as_db_str().into()],
            ))
            .await?;
        }
        Ok(())
    }
}

/// Removes coats whose zone no longer exists. They were left behind when a zone was
/// deleted while SQLite foreign keys were off; nothing displays or uses them.
pub struct OrphanLayerCleanupMigration;

impl MigrationName for OrphanLayerCleanupMigration {
    fn name(&self) -> &str {
        "m0004_orphan_layer_cleanup"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for OrphanLayerCleanupMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DELETE FROM project_layer WHERE zone_id NOT IN (SELECT id FROM project_zone);",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Deleted rows had no zone to come back to.
        Ok(())
    }
}
