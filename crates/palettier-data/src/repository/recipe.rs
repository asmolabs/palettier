use crate::db::{Database, map_db_err};
use crate::entity::{recipe, recipe_step};
use async_trait::async_trait;
use indexmap::IndexMap;
use palettier_domain::error::DomainResult;
use palettier_domain::paint::{LayerThickness, Medium, Technique};
use palettier_domain::port::RecipeRepository;
use palettier_domain::recipe::{Recipe, RecipeStep};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set, Statement, TransactionTrait,
};
use std::str::FromStr;

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
            steps_by_recipe
                .entry(s.recipe_id)
                .or_default()
                .push(RecipeStep {
                    id: Some(s.id),
                    technique: Technique::from_str(&s.technique).unwrap_or(Technique::BaseLayer),
                    paint_mix: s.paint_mix,
                    medium: Medium::from_str(&s.medium).unwrap_or(Medium::None),
                    medium_ratio: s.medium_ratio,
                    thickness: LayerThickness::from_str(&s.thickness)
                        .unwrap_or(LayerThickness::Normal),
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
        let count: i64 = row.map(|r| r.try_get("", "cnt").unwrap_or(0)).unwrap_or(0);
        Ok(count > 0)
    }

    async fn save(&self, rec: &Recipe) -> DomainResult<Recipe> {
        let txn = self.db.begin().await.map_err(map_db_err)?;
        let mut active = recipe::ActiveModel {
            name: Set(rec.name.clone()),
            subject: Set(rec.subject.clone()),
            notes: Set(rec.notes.clone()),
            ..Default::default()
        };
        // A recipe already saved is updated in place and its steps rewritten.
        let id = if let Some(id) = rec.id {
            active.id = Set(id);
            active.update(&txn).await.map_err(map_db_err)?;
            recipe_step::Entity::delete_many()
                .filter(recipe_step::Column::RecipeId.eq(id))
                .exec(&txn)
                .await
                .map_err(map_db_err)?;
            id
        } else {
            active.insert(&txn).await.map_err(map_db_err)?.id
        };

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
