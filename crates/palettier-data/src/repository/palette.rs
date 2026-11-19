use crate::db::{Database, DbExec, map_db_err};
use crate::entity::{palette, palette_paint};
use async_trait::async_trait;
use indexmap::{IndexMap, IndexSet};
use palettier_domain::error::DomainResult;
use palettier_domain::paint::Paint;
use palettier_domain::palette::Palette;
use palettier_domain::port::{PaintCatalogRepository, PaletteRepository};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct SeaOrmPaletteRepository {
    db: DbExec,
    catalog: Arc<dyn PaintCatalogRepository>,
}

impl SeaOrmPaletteRepository {
    pub fn new(database: &Database, catalog: Arc<dyn PaintCatalogRepository>) -> Self {
        Self {
            db: DbExec::Pool(database.connection().clone()),
            catalog,
        }
    }

    pub(crate) fn from_exec(db: DbExec, catalog: Arc<dyn PaintCatalogRepository>) -> Self {
        Self { db, catalog }
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
            links_by_palette
                .entry(l.palette_id)
                .or_default()
                .push(l.paint_id);
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
