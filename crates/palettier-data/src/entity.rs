pub mod paint {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "paint")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub brand: String,
        pub name: String,
        pub code: String,
        pub legacy_code: String,
        pub hex_color: String,
        pub tint_hex: Option<String>,
        pub opacity: String,
        pub drying_class: String,
        pub tinting_strength: f64,
        pub colour_derived: i64,
        pub pigments_verified: i64,
        pub user_added: i64,
        pub in_stock: i64,
        pub notes: String,
        pub paint_system: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::paint_pigment::Entity")]
        Pigments,
    }

    impl Related<super::paint_pigment::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Pigments.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod paint_pigment {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "paint_pigment")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub paint_id: i64,
        #[sea_orm(primary_key, auto_increment = false)]
        pub pigment: String,
        pub position: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::paint::Entity",
            from = "Column::PaintId",
            to = "super::paint::Column::Id",
            on_delete = "Cascade"
        )]
        Paint,
    }

    impl Related<super::paint::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Paint.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod palette {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "palette")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub name: String,
        pub purpose: String,
        pub notes: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::palette_paint::Entity")]
        Paints,
    }

    impl Related<super::palette_paint::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Paints.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod palette_paint {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "palette_paint")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub palette_id: i64,
        #[sea_orm(primary_key, auto_increment = false)]
        pub paint_id: i64,
        pub position: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::palette::Entity",
            from = "Column::PaletteId",
            to = "super::palette::Column::Id",
            on_delete = "Cascade"
        )]
        Palette,
        #[sea_orm(
            belongs_to = "super::paint::Entity",
            from = "Column::PaintId",
            to = "super::paint::Column::Id",
            on_delete = "Cascade"
        )]
        Paint,
    }

    impl Related<super::palette::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Palette.def()
        }
    }

    impl Related<super::paint::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Paint.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod project {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "project")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub name: String,
        pub subject: String,
        pub approach: String,
        pub notes: String,
        pub palette_id: Option<i64>,
        pub created_at: i64,
        pub paint_system: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::project_paint::Entity")]
        Paints,
        #[sea_orm(has_many = "super::project_zone::Entity")]
        Zones,
        #[sea_orm(has_many = "super::project_photo::Entity")]
        Photos,
    }

    impl Related<super::project_paint::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Paints.def()
        }
    }

    impl Related<super::project_zone::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Zones.def()
        }
    }

    impl Related<super::project_photo::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Photos.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod project_paint {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "project_paint")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub project_id: i64,
        #[sea_orm(primary_key, auto_increment = false)]
        pub paint_id: i64,
        pub position: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::project::Entity",
            from = "Column::ProjectId",
            to = "super::project::Column::Id",
            on_delete = "Cascade"
        )]
        Project,
        #[sea_orm(
            belongs_to = "super::paint::Entity",
            from = "Column::PaintId",
            to = "super::paint::Column::Id",
            on_delete = "Cascade"
        )]
        Paint,
    }

    impl Related<super::project::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Project.def()
        }
    }

    impl Related<super::paint::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Paint.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod project_zone {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "project_zone")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub project_id: i64,
        pub name: String,
        pub material: String,
        pub note: String,
        pub position: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::project::Entity",
            from = "Column::ProjectId",
            to = "super::project::Column::Id",
            on_delete = "Cascade"
        )]
        Project,
        #[sea_orm(has_many = "super::project_layer::Entity")]
        Layers,
    }

    impl Related<super::project::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Project.def()
        }
    }

    impl Related<super::project_layer::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Layers.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod project_layer {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "project_layer")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub zone_id: i64,
        pub role: String,
        pub target_hex: String,
        pub technique: String,
        pub note: String,
        pub kind: String,
        pub position: i64,
        pub applied_at: Option<i64>,
        pub applied_temperature: Option<f64>,
        pub applied_humidity: Option<f64>,
        pub applied_ventilation: Option<String>,
        pub applied_drying_class: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::project_zone::Entity",
            from = "Column::ZoneId",
            to = "super::project_zone::Column::Id",
            on_delete = "Cascade"
        )]
        Zone,
    }

    impl Related<super::project_zone::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Zone.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod project_photo {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "project_photo")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub project_id: i64,
        pub data: Vec<u8>,
        pub role: String,
        pub caption: String,
        pub added_at: i64,
        pub position: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::project::Entity",
            from = "Column::ProjectId",
            to = "super::project::Column::Id",
            on_delete = "Cascade"
        )]
        Project,
    }

    impl Related<super::project::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Project.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod recipe {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "recipe")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub name: String,
        pub subject: String,
        pub notes: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::recipe_step::Entity")]
        Steps,
    }

    impl Related<super::recipe_step::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Steps.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod recipe_step {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "recipe_step")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub recipe_id: i64,
        pub technique: String,
        pub paint_mix: String,
        pub medium: String,
        pub medium_ratio: f64,
        pub thickness: String,
        pub notes: String,
        pub position: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::recipe::Entity",
            from = "Column::RecipeId",
            to = "super::recipe::Column::Id",
            on_delete = "Cascade"
        )]
        Recipe,
    }

    impl Related<super::recipe::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Recipe.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod palette_mix {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "palette_mix")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub name: String,
        pub hex_color: String,
        pub recipe: String,
        pub drying_class: String,
        pub mixed_at: i64,
        pub temperature: f64,
        pub humidity: f64,
        pub ventilation: String,
        pub paint_system: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod setting {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "setting")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub key: String,
        pub value: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
