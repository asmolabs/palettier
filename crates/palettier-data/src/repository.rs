use crate::db::{Database, map_db_err};
use crate::entity::{
    paint, paint_pigment, palette, palette_mix, palette_paint, project, project_layer,
    project_paint, project_photo, project_zone, recipe, recipe_step, setting,
};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use indexmap::{IndexMap, IndexSet};
use palettier_domain::color::Rgb;
use palettier_domain::drying::Workshop;
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::paint::{DryingClass, LayerThickness, Medium, Opacity, Paint, Technique, Ventilation};
use palettier_domain::palette::{Palette, PaletteMix};
use palettier_domain::port::{
    PaintCatalogRepository, PaletteMixRepository, PaletteRepository, ProjectRepository,
    RecipeRepository, SettingsRepository,
};
use palettier_domain::project::{
    AppliedCoat, LayerKind, PhotoRole, Project, ProjectLayer, ProjectPhoto, ProjectZone,
};
use palettier_domain::recipe::{Recipe, RecipeStep};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set, Statement, TransactionTrait,
};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
pub struct SeaOrmPaintCatalogRepository {
    db: DatabaseConnection,
}

impl SeaOrmPaintCatalogRepository {
    pub fn new(database: &Database) -> Self {
        Self {
            db: database.connection().clone(),
        }
    }

    async fn with_pigments(&self, rows: Vec<paint::Model>) -> DomainResult<Vec<Paint>> {
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let all_pigments = paint_pigment::Entity::find()
            .order_by_asc(paint_pigment::Column::PaintId)
            .order_by_asc(paint_pigment::Column::Position)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;

        let mut by_paint: IndexMap<i64, IndexSet<String>> = IndexMap::new();
        for p in all_pigments {
            by_paint.entry(p.paint_id).or_default().insert(p.pigment);
        }

        Ok(rows
            .into_iter()
            .map(|row| {
                let pigments = by_paint.swap_remove(&row.id).unwrap_or_default();
                row_to_paint(row, pigments)
            })
            .collect())
    }

    async fn pigments_of<C: ConnectionTrait>(&self, conn: &C, paint_id: i64) -> DomainResult<IndexSet<String>> {
        let rows = paint_pigment::Entity::find()
            .filter(paint_pigment::Column::PaintId.eq(paint_id))
            .order_by_asc(paint_pigment::Column::Position)
            .all(conn)
            .await
            .map_err(map_db_err)?;
        Ok(rows.into_iter().map(|r| r.pigment).collect())
    }
}

fn row_to_paint(row: paint::Model, pigments: IndexSet<String>) -> Paint {
    Paint {
        id: Some(row.id),
        brand: row.brand,
        name: row.name,
        code: row.code,
        legacy_code: row.legacy_code,
        pigments,
        hex_color: row.hex_color,
        tint_hex: row.tint_hex,
        opacity: Opacity::from_str(&row.opacity).unwrap_or(Opacity::SemiOpaque),
        drying_class: DryingClass::from_str(&row.drying_class).unwrap_or(DryingClass::Medium),
        tinting_strength: row.tinting_strength,
        color_derived: row.colour_derived != 0,
        pigments_verified: row.pigments_verified != 0,
        user_added: row.user_added != 0,
        in_stock: row.in_stock != 0,
        notes: row.notes,
    }
}

#[async_trait]
impl PaintCatalogRepository for SeaOrmPaintCatalogRepository {
    async fn all(&self) -> DomainResult<Vec<Paint>> {
        let rows = paint::Entity::find()
            .order_by_asc(paint::Column::Brand)
            .order_by_asc(paint::Column::Name)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;
        self.with_pigments(rows).await
    }

    async fn in_stock(&self) -> DomainResult<Vec<Paint>> {
        let rows = paint::Entity::find()
            .filter(paint::Column::InStock.eq(1))
            .order_by_asc(paint::Column::Brand)
            .order_by_asc(paint::Column::Name)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;
        self.with_pigments(rows).await
    }

    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Paint>> {
        let Some(row) = paint::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?
        else {
            return Ok(None);
        };
        let pigments = self.pigments_of(&self.db, row.id).await?;
        Ok(Some(row_to_paint(row, pigments)))
    }

    async fn find_by_natural_key(&self, brand: &str, name: &str) -> DomainResult<Option<Paint>> {
        let Some(row) = paint::Entity::find()
            .filter(paint::Column::Brand.eq(brand))
            .filter(paint::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(map_db_err)?
        else {
            return Ok(None);
        };
        let pigments = self.pigments_of(&self.db, row.id).await?;
        Ok(Some(row_to_paint(row, pigments)))
    }

    async fn save(&self, p: &Paint) -> DomainResult<Paint> {
        let txn = self.db.begin().await.map_err(map_db_err)?;
        let active = paint::ActiveModel {
            brand: Set(p.brand.clone()),
            name: Set(p.name.clone()),
            code: Set(p.code.clone()),
            legacy_code: Set(p.legacy_code.clone()),
            hex_color: Set(p.hex_color.clone()),
            tint_hex: Set(p.tint_hex.clone()),
            opacity: Set(p.opacity.as_db_str().to_string()),
            drying_class: Set(p.drying_class.as_db_str().to_string()),
            tinting_strength: Set(p.tinting_strength),
            colour_derived: Set(i64::from(p.color_derived)),
            pigments_verified: Set(i64::from(p.pigments_verified)),
            user_added: Set(i64::from(p.user_added)),
            in_stock: Set(i64::from(p.in_stock)),
            notes: Set(p.notes.clone()),
            ..Default::default()
        };
        let inserted = active.insert(&txn).await.map_err(map_db_err)?;
        let id = inserted.id;
        for (pos, code) in p.pigments.iter().enumerate() {
            let pig = paint_pigment::ActiveModel {
                paint_id: Set(id),
                pigment: Set(code.clone()),
                position: Set(pos as i64),
            };
            let _ = paint_pigment::Entity::insert(pig)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        paint_pigment::Column::PaintId,
                        paint_pigment::Column::Pigment,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec(&txn)
                .await;
        }
        txn.commit().await.map_err(map_db_err)?;
        let mut out = p.clone();
        out.id = Some(id);
        Ok(out)
    }

    async fn save_all(&self, paints: &[Paint]) -> DomainResult<()> {
        let txn = self.db.begin().await.map_err(map_db_err)?;
        for p in paints {
            let active = paint::ActiveModel {
                brand: Set(p.brand.clone()),
                name: Set(p.name.clone()),
                code: Set(p.code.clone()),
                legacy_code: Set(p.legacy_code.clone()),
                hex_color: Set(p.hex_color.clone()),
                tint_hex: Set(p.tint_hex.clone()),
                opacity: Set(p.opacity.as_db_str().to_string()),
                drying_class: Set(p.drying_class.as_db_str().to_string()),
                tinting_strength: Set(p.tinting_strength),
                colour_derived: Set(i64::from(p.color_derived)),
                pigments_verified: Set(i64::from(p.pigments_verified)),
                user_added: Set(i64::from(p.user_added)),
                in_stock: Set(i64::from(p.in_stock)),
                notes: Set(p.notes.clone()),
                ..Default::default()
            };
            let inserted = active.insert(&txn).await.map_err(map_db_err)?;
            let id = inserted.id;
            for (pos, code) in p.pigments.iter().enumerate() {
                let pig = paint_pigment::ActiveModel {
                    paint_id: Set(id),
                    pigment: Set(code.clone()),
                    position: Set(pos as i64),
                };
                let _ = paint_pigment::Entity::insert(pig)
                    .on_conflict(
                        sea_orm::sea_query::OnConflict::columns([
                            paint_pigment::Column::PaintId,
                            paint_pigment::Column::Pigment,
                        ])
                        .do_nothing()
                        .to_owned(),
                    )
                    .exec(&txn)
                    .await;
            }
        }
        txn.commit().await.map_err(map_db_err)?;
        Ok(())
    }

    async fn set_owned(&self, paint_id: i64, owned: bool) -> DomainResult<()> {
        let Some(row) = paint::Entity::find_by_id(paint_id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?
        else {
            return Err(DomainError::InvalidArgument(format!("Tube introuvable: {paint_id}")));
        };
        let mut active: paint::ActiveModel = row.into();
        active.in_stock = Set(i64::from(owned));
        active.update(&self.db).await.map_err(map_db_err)?;
        Ok(())
    }

    async fn record_tint(&self, paint_id: i64, tint: Rgb) -> DomainResult<()> {
        let Some(row) = paint::Entity::find_by_id(paint_id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?
        else {
            return Err(DomainError::InvalidArgument(format!("Tube introuvable: {paint_id}")));
        };
        let mut active: paint::ActiveModel = row.into();
        active.tint_hex = Set(Some(tint.to_hex()));
        active.update(&self.db).await.map_err(map_db_err)?;
        Ok(())
    }

    async fn record_masstone(&self, paint_id: i64, measured: Rgb) -> DomainResult<()> {
        let Some(row) = paint::Entity::find_by_id(paint_id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?
        else {
            return Err(DomainError::InvalidArgument(format!("Tube introuvable: {paint_id}")));
        };
        let mut active: paint::ActiveModel = row.into();
        active.hex_color = Set(measured.to_hex());
        active.colour_derived = Set(0);
        active.update(&self.db).await.map_err(map_db_err)?;
        Ok(())
    }

    async fn count_owned(&self) -> DomainResult<u64> {
        paint::Entity::find()
            .filter(paint::Column::InStock.eq(1))
            .count(&self.db)
            .await
            .map_err(map_db_err)
    }

    async fn count(&self) -> DomainResult<u64> {
        paint::Entity::find().count(&self.db).await.map_err(map_db_err)
    }

    async fn refresh_identity(&self, p: &Paint) -> DomainResult<()> {
        let id = p
            .id
            .ok_or_else(|| DomainError::InvalidArgument("Tube sans identifiant".to_string()))?;
        let txn = self.db.begin().await.map_err(map_db_err)?;
        let Some(row) = paint::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(map_db_err)?
        else {
            return Err(DomainError::InvalidArgument(format!("Tube introuvable: {id}")));
        };
        let mut active: paint::ActiveModel = row.into();
        active.code = Set(p.code.clone());
        active.legacy_code = Set(p.legacy_code.clone());
        active.opacity = Set(p.opacity.as_db_str().to_string());
        active.drying_class = Set(p.drying_class.as_db_str().to_string());
        active.tinting_strength = Set(p.tinting_strength);
        active.pigments_verified = Set(i64::from(p.pigments_verified));
        active.hex_color = Set(p.hex_color.clone());
        active.colour_derived = Set(i64::from(p.color_derived));
        active.update(&txn).await.map_err(map_db_err)?;

        paint_pigment::Entity::delete_many()
            .filter(paint_pigment::Column::PaintId.eq(id))
            .exec(&txn)
            .await
            .map_err(map_db_err)?;

        for (pos, code) in p.pigments.iter().enumerate() {
            let pig = paint_pigment::ActiveModel {
                paint_id: Set(id),
                pigment: Set(code.clone()),
                position: Set(pos as i64),
            };
            let _ = paint_pigment::Entity::insert(pig)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        paint_pigment::Column::PaintId,
                        paint_pigment::Column::Pigment,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec(&txn)
                .await;
        }
        txn.commit().await.map_err(map_db_err)?;
        Ok(())
    }

    async fn ids_in_use(&self) -> DomainResult<IndexSet<i64>> {
        let pal_links = palette_paint::Entity::find()
            .all(&self.db)
            .await
            .map_err(map_db_err)?;
        let proj_links = project_paint::Entity::find()
            .all(&self.db)
            .await
            .map_err(map_db_err)?;
        let mut set = IndexSet::new();
        for l in pal_links {
            set.insert(l.paint_id);
        }
        for l in proj_links {
            set.insert(l.paint_id);
        }
        Ok(set)
    }

    async fn delete(&self, paint_id: i64) -> DomainResult<()> {
        paint::Entity::delete_by_id(paint_id)
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct SeaOrmPaletteRepository {
    db: DatabaseConnection,
    catalog: Arc<dyn PaintCatalogRepository>,
}

impl SeaOrmPaletteRepository {
    pub fn new(database: &Database, catalog: Arc<dyn PaintCatalogRepository>) -> Self {
        Self {
            db: database.connection().clone(),
            catalog,
        }
    }

    async fn assemble(&self, rows: Vec<palette::Model>) -> DomainResult<Vec<Palette>> {
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let all_paints = self.catalog.all().await?;
        let by_id: IndexMap<i64, Paint> = all_paints
            .into_iter()
            .filter_map(|p| p.id.map(|id| (id, p)))
            .collect();

        let links = palette_paint::Entity::find()
            .order_by_asc(palette_paint::Column::PaletteId)
            .order_by_asc(palette_paint::Column::Position)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;

        let mut links_by_palette: IndexMap<i64, Vec<i64>> = IndexMap::new();
        for l in links {
            links_by_palette.entry(l.palette_id).or_default().push(l.paint_id);
        }

        Ok(rows
            .into_iter()
            .map(|row| {
                let paints = links_by_palette
                    .get(&row.id)
                    .map(|ids| ids.iter().filter_map(|id| by_id.get(id).cloned()).collect())
                    .unwrap_or_default();
                Palette {
                    id: Some(row.id),
                    name: row.name,
                    purpose: row.purpose,
                    notes: row.notes,
                    paints,
                }
            })
            .collect())
    }
}

#[async_trait]
impl PaletteRepository for SeaOrmPaletteRepository {
    async fn all(&self) -> DomainResult<Vec<Palette>> {
        let rows = palette::Entity::find()
            .order_by_asc(palette::Column::Name)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;
        self.assemble(rows).await
    }

    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Palette>> {
        let Some(row) = palette::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?
        else {
            return Ok(None);
        };
        Ok(self.assemble(vec![row]).await?.into_iter().next())
    }

    async fn save(&self, pal: &Palette) -> DomainResult<Palette> {
        let txn = self.db.begin().await.map_err(map_db_err)?;
        let id = if let Some(existing_id) = pal.id {
            let active = palette::ActiveModel {
                id: Set(existing_id),
                name: Set(pal.name.clone()),
                purpose: Set(pal.purpose.clone()),
                notes: Set(pal.notes.clone()),
            };
            active.update(&txn).await.map_err(map_db_err)?;
            existing_id
        } else {
            let active = palette::ActiveModel {
                name: Set(pal.name.clone()),
                purpose: Set(pal.purpose.clone()),
                notes: Set(pal.notes.clone()),
                ..Default::default()
            };
            active.insert(&txn).await.map_err(map_db_err)?.id
        };

        palette_paint::Entity::delete_many()
            .filter(palette_paint::Column::PaletteId.eq(id))
            .exec(&txn)
            .await
            .map_err(map_db_err)?;

        let mut seen = IndexSet::new();
        for (pos, p) in pal.paints.iter().enumerate() {
            if let Some(paint_id) = p.id
                && seen.insert(paint_id)
            {
                let link = palette_paint::ActiveModel {
                    palette_id: Set(id),
                    paint_id: Set(paint_id),
                    position: Set(pos as i64),
                };
                link.insert(&txn).await.map_err(map_db_err)?;
            }
        }
        txn.commit().await.map_err(map_db_err)?;
        let mut out = pal.clone();
        out.id = Some(id);
        Ok(out)
    }

    async fn delete(&self, id: i64) -> DomainResult<()> {
        palette::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct SeaOrmProjectRepository {
    db: DatabaseConnection,
    catalog: Arc<dyn PaintCatalogRepository>,
    palettes: Arc<dyn PaletteRepository>,
}

impl SeaOrmProjectRepository {
    pub fn new(
        database: &Database,
        catalog: Arc<dyn PaintCatalogRepository>,
        palettes: Arc<dyn PaletteRepository>,
    ) -> Self {
        Self {
            db: database.connection().clone(),
            catalog,
            palettes,
        }
    }

    async fn unique_name<C: ConnectionTrait>(&self, conn: &C, wanted: &str) -> DomainResult<String> {
        let base = if wanted.trim().is_empty() {
            "Projet"
        } else {
            wanted.trim()
        };
        let mut candidate = base.to_string();
        let mut suffix = 2;
        while self.exists_by_name(conn, &candidate).await? {
            candidate = format!("{base} {suffix}");
            suffix += 1;
        }
        Ok(candidate)
    }

    async fn exists_by_name<C: ConnectionTrait>(&self, conn: &C, name: &str) -> DomainResult<bool> {
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT count(*) AS cnt FROM project WHERE name = ? COLLATE NOCASE",
            [name.into()],
        );
        let row = conn.query_one_raw(stmt).await.map_err(map_db_err)?;
        let count: i64 = row
            .map(|r| r.try_get("", "cnt").unwrap_or(0))
            .unwrap_or(0);
        Ok(count > 0)
    }

    async fn assemble(
        &self,
        rows: Vec<project::Model>,
        mut photos_map: IndexMap<i64, Vec<ProjectPhoto>>,
    ) -> DomainResult<Vec<Project>> {
        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let paints_by_id: IndexMap<i64, Paint> = self
            .catalog
            .all()
            .await?
            .into_iter()
            .filter_map(|p| p.id.map(|id| (id, p)))
            .collect();

        let palettes_by_id: IndexMap<i64, Palette> = self
            .palettes
            .all()
            .await?
            .into_iter()
            .filter_map(|p| p.id.map(|id| (id, p)))
            .collect();

        let frozen_rows = project_paint::Entity::find()
            .order_by_asc(project_paint::Column::ProjectId)
            .order_by_asc(project_paint::Column::Position)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;
        let mut frozen: IndexMap<i64, Vec<i64>> = IndexMap::new();
        for f in frozen_rows {
            frozen.entry(f.project_id).or_default().push(f.paint_id);
        }

        let zone_rows = project_zone::Entity::find()
            .order_by_asc(project_zone::Column::ProjectId)
            .order_by_asc(project_zone::Column::Position)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;
        let mut zones_by_project: IndexMap<i64, Vec<project_zone::Model>> = IndexMap::new();
        for z in zone_rows {
            zones_by_project.entry(z.project_id).or_default().push(z);
        }

        let layer_rows = project_layer::Entity::find()
            .order_by_asc(project_layer::Column::ZoneId)
            .order_by_asc(project_layer::Column::Position)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;
        let mut layers_by_zone: IndexMap<i64, Vec<project_layer::Model>> = IndexMap::new();
        for l in layer_rows {
            layers_by_zone.entry(l.zone_id).or_default().push(l);
        }

        Ok(rows
            .into_iter()
            .map(|row| {
                let paints = frozen
                    .get(&row.id)
                    .map(|ids| ids.iter().filter_map(|id| paints_by_id.get(id).cloned()).collect())
                    .unwrap_or_default();
                let zones = zones_by_project
                    .swap_remove(&row.id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|z| {
                        let layers = layers_by_zone
                            .swap_remove(&z.id)
                            .unwrap_or_default()
                            .into_iter()
                            .map(layer_row_to_domain)
                            .collect();
                        ProjectZone {
                            id: Some(z.id),
                            name: z.name,
                            material: z.material,
                            note: z.note,
                            layers,
                        }
                    })
                    .collect();
                let photos = photos_map.swap_remove(&row.id).unwrap_or_default();
                let created_at = Utc
                    .timestamp_millis_opt(row.created_at)
                    .single()
                    .unwrap_or_else(Utc::now);

                Project {
                    id: Some(row.id),
                    name: row.name,
                    subject: row.subject,
                    approach: row.approach,
                    notes: row.notes,
                    palette: row.palette_id.and_then(|pid| palettes_by_id.get(&pid).cloned()),
                    paints,
                    zones,
                    photos,
                    created_at,
                }
            })
            .collect())
    }
}

fn layer_row_to_domain(row: project_layer::Model) -> ProjectLayer {
    let std_workshop = Workshop::standard();
    let applied = row.applied_at.map(|ms| {
        let at = Utc.timestamp_millis_opt(ms).single().unwrap_or_else(Utc::now);
        let temp = row.applied_temperature.unwrap_or(std_workshop.temperature_celsius);
        let hum = row.applied_humidity.unwrap_or(std_workshop.relative_humidity);
        let vent = row
            .applied_ventilation
            .as_deref()
            .and_then(|s| Ventilation::from_str(s).ok())
            .unwrap_or(std_workshop.ventilation);
        let dc = row
            .applied_drying_class
            .as_deref()
            .and_then(|s| DryingClass::from_str(s).ok())
            .unwrap_or(DryingClass::Medium);
        AppliedCoat {
            at,
            workshop: Workshop::new(temp, hum, vent).unwrap_or(std_workshop),
            drying_class: dc,
        }
    });

    ProjectLayer {
        id: Some(row.id),
        role: row.role,
        target_hex: row.target_hex,
        technique: row.technique,
        note: row.note,
        kind: LayerKind::from_str(&row.kind).unwrap_or(LayerKind::Ladder),
        applied,
    }
}

#[async_trait]
impl ProjectRepository for SeaOrmProjectRepository {
    async fn all(&self) -> DomainResult<Vec<Project>> {
        let rows = project::Entity::find()
            .order_by_desc(project::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;
        self.assemble(rows, IndexMap::new()).await
    }

    async fn find_with_photos(&self, id: i64) -> DomainResult<Option<Project>> {
        let Some(row) = project::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?
        else {
            return Ok(None);
        };
        let photo_rows = project_photo::Entity::find()
            .filter(project_photo::Column::ProjectId.eq(id))
            .order_by_asc(project_photo::Column::Position)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;

        let photos: Vec<ProjectPhoto> = photo_rows
            .into_iter()
            .map(|p| ProjectPhoto {
                id: Some(p.id),
                data: p.data,
                role: PhotoRole::from_str(&p.role).unwrap_or(PhotoRole::Piece),
                caption: p.caption,
                added_at: Utc
                    .timestamp_millis_opt(p.added_at)
                    .single()
                    .unwrap_or_else(Utc::now),
            })
            .collect();

        let mut map = IndexMap::new();
        map.insert(id, photos);
        Ok(self.assemble(vec![row], map).await?.into_iter().next())
    }

    async fn save(&self, proj: &Project) -> DomainResult<Project> {
        let txn = self.db.begin().await.map_err(map_db_err)?;
        let (id, saved_name) = if let Some(existing_id) = proj.id {
            let active = project::ActiveModel {
                id: Set(existing_id),
                name: Set(proj.name.clone()),
                subject: Set(proj.subject.clone()),
                approach: Set(proj.approach.clone()),
                notes: Set(proj.notes.clone()),
                palette_id: Set(proj.palette.as_ref().and_then(|p| p.id)),
                created_at: Set(proj.created_at.timestamp_millis()),
            };
            active.update(&txn).await.map_err(map_db_err)?;
            (existing_id, proj.name.clone())
        } else {
            let uname = self.unique_name(&txn, &proj.name).await?;
            let active = project::ActiveModel {
                name: Set(uname.clone()),
                subject: Set(proj.subject.clone()),
                approach: Set(proj.approach.clone()),
                notes: Set(proj.notes.clone()),
                palette_id: Set(proj.palette.as_ref().and_then(|p| p.id)),
                created_at: Set(proj.created_at.timestamp_millis()),
                ..Default::default()
            };
            let inserted = active.insert(&txn).await.map_err(map_db_err)?;
            (inserted.id, uname)
        };

        project_paint::Entity::delete_many()
            .filter(project_paint::Column::ProjectId.eq(id))
            .exec(&txn)
            .await
            .map_err(map_db_err)?;

        let mut seen_paints = IndexSet::new();
        for (pos, paint) in proj.paints.iter().enumerate() {
            if let Some(paint_id) = paint.id
                && seen_paints.insert(paint_id)
            {
                let link = project_paint::ActiveModel {
                    project_id: Set(id),
                    paint_id: Set(paint_id),
                    position: Set(pos as i64),
                };
                link.insert(&txn).await.map_err(map_db_err)?;
            }
        }

        // NEVER touch project_photo!
        project_zone::Entity::delete_many()
            .filter(project_zone::Column::ProjectId.eq(id))
            .exec(&txn)
            .await
            .map_err(map_db_err)?;

        let mut saved_zones = Vec::with_capacity(proj.zones.len());
        for (zone_pos, zone) in proj.zones.iter().enumerate() {
            let z_active = project_zone::ActiveModel {
                project_id: Set(id),
                name: Set(zone.name.clone()),
                material: Set(zone.material.clone()),
                note: Set(zone.note.clone()),
                position: Set(zone_pos as i64),
                ..Default::default()
            };
            let z_inserted = z_active.insert(&txn).await.map_err(map_db_err)?;
            let zone_id = z_inserted.id;

            let mut saved_layers = Vec::with_capacity(zone.layers.len());
            for (layer_pos, layer) in zone.layers.iter().enumerate() {
                let l_active = project_layer::ActiveModel {
                    zone_id: Set(zone_id),
                    role: Set(layer.role.clone()),
                    target_hex: Set(layer.target_hex.clone()),
                    technique: Set(layer.technique.clone()),
                    note: Set(layer.note.clone()),
                    kind: Set(layer.kind.as_db_str().to_string()),
                    position: Set(layer_pos as i64),
                    applied_at: Set(layer.applied.as_ref().map(|a| a.at.timestamp_millis())),
                    applied_temperature: Set(layer
                        .applied
                        .as_ref()
                        .map(|a| a.workshop.temperature_celsius)),
                    applied_humidity: Set(layer
                        .applied
                        .as_ref()
                        .map(|a| a.workshop.relative_humidity)),
                    applied_ventilation: Set(layer
                        .applied
                        .as_ref()
                        .map(|a| a.workshop.ventilation.as_db_str().to_string())),
                    applied_drying_class: Set(layer
                        .applied
                        .as_ref()
                        .map(|a| a.drying_class.as_db_str().to_string())),
                    ..Default::default()
                };
                let l_inserted = l_active.insert(&txn).await.map_err(map_db_err)?;
                let mut l_clone = layer.clone();
                l_clone.id = Some(l_inserted.id);
                saved_layers.push(l_clone);
            }
            let mut z_clone = zone.clone();
            z_clone.id = Some(zone_id);
            z_clone.layers = saved_layers;
            saved_zones.push(z_clone);
        }

        txn.commit().await.map_err(map_db_err)?;
        let mut out = proj.clone();
        out.id = Some(id);
        out.name = saved_name;
        out.zones = saved_zones;
        Ok(out)
    }

    async fn add_photo(&self, project_id: i64, photo: &ProjectPhoto) -> DomainResult<Project> {
        let txn = self.db.begin().await.map_err(map_db_err)?;
        let existing_count = project_photo::Entity::find()
            .filter(project_photo::Column::ProjectId.eq(project_id))
            .count(&txn)
            .await
            .map_err(map_db_err)?;

        let active = project_photo::ActiveModel {
            project_id: Set(project_id),
            data: Set(photo.data.clone()),
            role: Set(photo.role.as_db_str().to_string()),
            caption: Set(photo.caption.clone()),
            added_at: Set(photo.added_at.timestamp_millis()),
            position: Set(existing_count as i64),
            ..Default::default()
        };
        active.insert(&txn).await.map_err(map_db_err)?;
        txn.commit().await.map_err(map_db_err)?;

        self.find_with_photos(project_id)
            .await?
            .ok_or_else(|| DomainError::InvalidArgument(format!("Projet introuvable: {project_id}")))
    }

    async fn remove_photo(&self, project_id: i64, photo_id: i64) -> DomainResult<Project> {
        project_photo::Entity::delete_by_id(photo_id)
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        self.find_with_photos(project_id)
            .await?
            .ok_or_else(|| DomainError::InvalidArgument(format!("Projet introuvable: {project_id}")))
    }

    async fn delete(&self, project_id: i64) -> DomainResult<()> {
        project::Entity::delete_by_id(project_id)
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct SeaOrmPaletteMixRepository {
    db: DatabaseConnection,
}

impl SeaOrmPaletteMixRepository {
    pub fn new(database: &Database) -> Self {
        Self {
            db: database.connection().clone(),
        }
    }
}

fn mix_row_to_domain(row: palette_mix::Model) -> PaletteMix {
    let std_w = Workshop::standard();
    let vent = Ventilation::from_str(&row.ventilation).unwrap_or(std_w.ventilation);
    let workshop = Workshop::new(row.temperature, row.humidity, vent).unwrap_or(std_w);
    PaletteMix {
        id: Some(row.id),
        name: row.name,
        hex_color: row.hex_color,
        recipe: row.recipe,
        drying_class: DryingClass::from_str(&row.drying_class).unwrap_or(DryingClass::Medium),
        mixed_at: Utc
            .timestamp_millis_opt(row.mixed_at)
            .single()
            .unwrap_or_else(Utc::now),
        workshop,
    }
}

#[async_trait]
impl PaletteMixRepository for SeaOrmPaletteMixRepository {
    async fn all(&self) -> DomainResult<Vec<PaletteMix>> {
        let rows = palette_mix::Entity::find()
            .order_by_desc(palette_mix::Column::MixedAt)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(rows.into_iter().map(mix_row_to_domain).collect())
    }

    async fn save(&self, mix: &PaletteMix) -> DomainResult<PaletteMix> {
        let active = palette_mix::ActiveModel {
            name: Set(mix.name.clone()),
            hex_color: Set(mix.hex_color.clone()),
            recipe: Set(mix.recipe.clone()),
            drying_class: Set(mix.drying_class.as_db_str().to_string()),
            mixed_at: Set(mix.mixed_at.timestamp_millis()),
            temperature: Set(mix.workshop.temperature_celsius),
            humidity: Set(mix.workshop.relative_humidity),
            ventilation: Set(mix.workshop.ventilation.as_db_str().to_string()),
            ..Default::default()
        };
        let inserted = active.insert(&self.db).await.map_err(map_db_err)?;
        let mut out = mix.clone();
        out.id = Some(inserted.id);
        Ok(out)
    }

    async fn delete(&self, id: i64) -> DomainResult<()> {
        palette_mix::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct SeaOrmRecipeRepository {
    db: DatabaseConnection,
}

impl SeaOrmRecipeRepository {
    pub fn new(database: &Database) -> Self {
        Self {
            db: database.connection().clone(),
        }
    }
}

#[async_trait]
impl RecipeRepository for SeaOrmRecipeRepository {
    async fn all(&self) -> DomainResult<Vec<Recipe>> {
        let rows = recipe::Entity::find()
            .order_by_asc(recipe::Column::Name)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let step_rows = recipe_step::Entity::find()
            .order_by_asc(recipe_step::Column::RecipeId)
            .order_by_asc(recipe_step::Column::Position)
            .all(&self.db)
            .await
            .map_err(map_db_err)?;

        let mut steps_by_recipe: IndexMap<i64, Vec<RecipeStep>> = IndexMap::new();
        for s in step_rows {
            steps_by_recipe.entry(s.recipe_id).or_default().push(RecipeStep {
                id: Some(s.id),
                technique: Technique::from_str(&s.technique).unwrap_or(Technique::BaseLayer),
                paint_mix: s.paint_mix,
                medium: Medium::from_str(&s.medium).unwrap_or(Medium::None),
                medium_ratio: s.medium_ratio,
                thickness: LayerThickness::from_str(&s.thickness).unwrap_or(LayerThickness::Normal),
                notes: s.notes,
            });
        }

        Ok(rows
            .into_iter()
            .map(|r| Recipe {
                id: Some(r.id),
                name: r.name,
                subject: r.subject,
                notes: r.notes,
                steps: steps_by_recipe.swap_remove(&r.id).unwrap_or_default(),
            })
            .collect())
    }

    async fn exists(&self, name: &str) -> DomainResult<bool> {
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT count(*) AS cnt FROM recipe WHERE name = ? COLLATE NOCASE",
            [name.into()],
        );
        let row = self.db.query_one_raw(stmt).await.map_err(map_db_err)?;
        let count: i64 = row
            .map(|r| r.try_get("", "cnt").unwrap_or(0))
            .unwrap_or(0);
        Ok(count > 0)
    }

    async fn save(&self, rec: &Recipe) -> DomainResult<Recipe> {
        let txn = self.db.begin().await.map_err(map_db_err)?;
        let active = recipe::ActiveModel {
            name: Set(rec.name.clone()),
            subject: Set(rec.subject.clone()),
            notes: Set(rec.notes.clone()),
            ..Default::default()
        };
        let inserted = active.insert(&txn).await.map_err(map_db_err)?;
        let id = inserted.id;

        for (pos, step) in rec.steps.iter().enumerate() {
            let s_active = recipe_step::ActiveModel {
                recipe_id: Set(id),
                technique: Set(step.technique.as_db_str().to_string()),
                paint_mix: Set(step.paint_mix.clone()),
                medium: Set(step.medium.as_db_str().to_string()),
                medium_ratio: Set(step.medium_ratio),
                thickness: Set(step.thickness.as_db_str().to_string()),
                notes: Set(step.notes.clone()),
                position: Set(pos as i64),
                ..Default::default()
            };
            s_active.insert(&txn).await.map_err(map_db_err)?;
        }
        txn.commit().await.map_err(map_db_err)?;
        let mut out = rec.clone();
        out.id = Some(id);
        Ok(out)
    }

    async fn delete(&self, id: i64) -> DomainResult<()> {
        recipe::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct SeaOrmSettingsRepository {
    db: DatabaseConnection,
    use_keyring: bool,
}

impl SeaOrmSettingsRepository {
    const KEYRING_SERVICE: &'static str = "be.asmolabs.palettier";

    pub fn new(database: &Database) -> Self {
        Self {
            db: database.connection().clone(),
            use_keyring: false,
        }
    }

    pub fn with_keyring(database: &Database) -> Self {
        Self {
            db: database.connection().clone(),
            use_keyring: true,
        }
    }

    fn is_secret_key(key: &str) -> bool {
        key.ends_with(".apiKey") || key.ends_with(".api_key")
    }
}

#[async_trait]
impl SettingsRepository for SeaOrmSettingsRepository {
    async fn get(&self, key: &str) -> DomainResult<Option<String>> {
        if self.use_keyring
            && Self::is_secret_key(key)
            && let Ok(entry) = keyring::Entry::new(Self::KEYRING_SERVICE, key)
            && let Ok(secret) = entry.get_password()
            && !secret.is_empty()
        {
            return Ok(Some(secret));
        }
        let row = setting::Entity::find_by_id(key.to_string())
            .one(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(row.map(|r| r.value))
    }

    async fn put(&self, key: &str, value: &str) -> DomainResult<()> {
        if self.use_keyring
            && Self::is_secret_key(key)
            && let Ok(entry) = keyring::Entry::new(Self::KEYRING_SERVICE, key)
            && entry.set_password(value).is_ok()
        {
            // Remove any plaintext key from the SQLite table
            let _ = setting::Entity::delete_by_id(key.to_string())
                .exec(&self.db)
                .await;
            return Ok(());
        }
        let active = setting::ActiveModel {
            key: Set(key.to_string()),
            value: Set(value.to_string()),
        };
        setting::Entity::insert(active)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(setting::Column::Key)
                    .update_column(setting::Column::Value)
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }

    async fn remove(&self, key: &str) -> DomainResult<()> {
        if self.use_keyring
            && Self::is_secret_key(key)
            && let Ok(entry) = keyring::Entry::new(Self::KEYRING_SERVICE, key)
        {
            let _ = entry.delete_credential();
        }
        setting::Entity::delete_by_id(key.to_string())
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }
}
