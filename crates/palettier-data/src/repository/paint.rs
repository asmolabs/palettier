use crate::db::{Database, map_db_err};
use crate::entity::{paint, paint_pigment, palette_paint, project_paint};
use async_trait::async_trait;
use indexmap::{IndexMap, IndexSet};
use palettier_domain::color::Rgb;
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::paint::{DryingClass, Opacity, Paint, PaintSystem};
use palettier_domain::port::PaintCatalogRepository;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use std::str::FromStr;

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

    async fn pigments_of<C: ConnectionTrait>(
        &self,
        conn: &C,
        paint_id: i64,
    ) -> DomainResult<IndexSet<String>> {
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
        system: PaintSystem::from_str(&row.paint_system).unwrap_or_default(),
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
            paint_system: Set(p.system.as_db_str().to_string()),
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
                paint_system: Set(p.system.as_db_str().to_string()),
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
            return Err(DomainError::InvalidArgument(format!(
                "Tube introuvable: {paint_id}"
            )));
        };
        let mut active: paint::ActiveModel = row.into();
        active.in_stock = Set(i64::from(owned));
        active.update(&self.db).await.map_err(map_db_err)?;
        Ok(())
    }

    async fn set_owned_batch(&self, paint_ids: &[i64], owned: bool) -> DomainResult<()> {
        if paint_ids.is_empty() {
            return Ok(());
        }
        let txn = self.db.begin().await.map_err(map_db_err)?;
        for chunk in paint_ids.chunks(500) {
            paint::Entity::update_many()
                .col_expr(paint::Column::InStock, Expr::value(i64::from(owned)))
                .filter(paint::Column::Id.is_in(chunk.iter().copied()))
                .exec(&txn)
                .await
                .map_err(map_db_err)?;
        }
        txn.commit().await.map_err(map_db_err)?;
        Ok(())
    }

    async fn record_tint(&self, paint_id: i64, tint: Rgb) -> DomainResult<()> {
        let Some(row) = paint::Entity::find_by_id(paint_id)
            .one(&self.db)
            .await
            .map_err(map_db_err)?
        else {
            return Err(DomainError::InvalidArgument(format!(
                "Tube introuvable: {paint_id}"
            )));
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
            return Err(DomainError::InvalidArgument(format!(
                "Tube introuvable: {paint_id}"
            )));
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
        paint::Entity::find()
            .count(&self.db)
            .await
            .map_err(map_db_err)
    }

    async fn refresh_identity(&self, p: &Paint) -> DomainResult<()> {
        let id =
            p.id.ok_or_else(|| DomainError::InvalidArgument("Tube sans identifiant".to_string()))?;
        let txn = self.db.begin().await.map_err(map_db_err)?;
        let Some(row) = paint::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(map_db_err)?
        else {
            return Err(DomainError::InvalidArgument(format!(
                "Tube introuvable: {id}"
            )));
        };
        let mut active: paint::ActiveModel = row.into();
        active.code = Set(p.code.clone());
        active.legacy_code = Set(p.legacy_code.clone());
        active.opacity = Set(p.opacity.as_db_str().to_string());
        active.drying_class = Set(p.drying_class.as_db_str().to_string());
        active.tinting_strength = Set(p.tinting_strength);
        active.pigments_verified = Set(i64::from(p.pigments_verified));
        active.paint_system = Set(p.system.as_db_str().to_string());
        active.hex_color = Set(p.hex_color.clone());
        active.tint_hex = Set(p.tint_hex.clone());
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
