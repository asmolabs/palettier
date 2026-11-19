use crate::db::{Database, DbExec, map_db_err};
use crate::entity::{
    project, project_layer, project_paint, project_photo, project_zone,
};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use indexmap::{IndexMap, IndexSet};
use palettier_domain::drying::Workshop;
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::paint::{DryingClass, Paint, PaintSystem, Technique, Ventilation};
use palettier_domain::palette::Palette;
use palettier_domain::port::{PaintCatalogRepository, PaletteRepository, ProjectRepository};
use palettier_domain::project::{
    AppliedCoat, LayerKind, PhotoRole, Project, ProjectLayer, ProjectPhoto, ProjectZone,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set, Statement, TransactionTrait,
};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
pub struct SeaOrmProjectRepository {
    db: DbExec,
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
            db: DbExec::Pool(database.connection().clone()),
            catalog,
            palettes,
        }
    }

    pub(crate) fn from_exec(
        db: DbExec,
        catalog: Arc<dyn PaintCatalogRepository>,
        palettes: Arc<dyn PaletteRepository>,
    ) -> Self {
        Self {
            db,
            catalog,
            palettes,
        }
    }

    async fn unique_name<C: ConnectionTrait>(
        &self,
        conn: &C,
        wanted: &str,
    ) -> DomainResult<String> {
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
        let count: i64 = row.map(|r| r.try_get("", "cnt").unwrap_or(0)).unwrap_or(0);
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
                let system = PaintSystem::from_str(&row.paint_system).unwrap_or_default();
                let paints = frozen
                    .get(&row.id)
                    .map(|ids| {
                        ids.iter()
                            .filter_map(|id| paints_by_id.get(id).cloned())
                            .collect()
                    })
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
                            .map(|l| layer_row_to_domain(l, system))
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
                    system,
                    palette: row
                        .palette_id
                        .and_then(|pid| palettes_by_id.get(&pid).cloned()),
                    paints,
                    zones,
                    photos,
                    created_at,
                }
            })
            .collect())
    }
}

fn layer_row_to_domain(row: project_layer::Model, system: PaintSystem) -> ProjectLayer {
    let std_workshop = Workshop::standard();
    let applied = row.applied_at.map(|ms| {
        let at = Utc
            .timestamp_millis_opt(ms)
            .single()
            .unwrap_or_else(Utc::now);
        let temp = row
            .applied_temperature
            .unwrap_or(std_workshop.temperature_celsius);
        let hum = row
            .applied_humidity
            .unwrap_or(std_workshop.relative_humidity);
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

    let mut layer = ProjectLayer {
        id: Some(row.id),
        role: row.role,
        target_hex: row.target_hex,
        technique: Technique::default_for(system),
        note: row.note,
        kind: LayerKind::from_str(&row.kind).unwrap_or(LayerKind::Ladder),
        applied,
    };
    // Stored as a code since migration m0003; anything else is read like a label.
    layer.set_technique_label(&row.technique, system);
    layer
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
                paint_system: Set(proj.system.as_db_str().to_string()),
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
                paint_system: Set(proj.system.as_db_str().to_string()),
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
                    technique: Set(layer.technique.as_db_str().to_string()),
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

        self.find_with_photos(project_id).await?.ok_or_else(|| {
            DomainError::InvalidArgument(format!("Projet introuvable: {project_id}"))
        })
    }

    async fn remove_photo(&self, project_id: i64, photo_id: i64) -> DomainResult<Project> {
        project_photo::Entity::delete_by_id(photo_id)
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        self.find_with_photos(project_id).await?.ok_or_else(|| {
            DomainError::InvalidArgument(format!("Projet introuvable: {project_id}"))
        })
    }

    async fn delete(&self, project_id: i64) -> DomainResult<()> {
        project::Entity::delete_by_id(project_id)
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }
}
