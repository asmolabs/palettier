use crate::db::{Database, map_db_err};
use crate::entity::palette_mix;
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use palettier_domain::drying::Workshop;
use palettier_domain::error::DomainResult;
use palettier_domain::paint::{DryingClass, PaintSystem, Ventilation};
use palettier_domain::palette::PaletteMix;
use palettier_domain::port::PaletteMixRepository;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryOrder, Set};
use std::str::FromStr;

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
        system: PaintSystem::from_str(&row.paint_system).unwrap_or_default(),
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
            paint_system: Set(mix.system.as_db_str().to_string()),
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
