--liquibase formatted sql
--
-- Rattrapage des bases nees avant Liquibase.
--
-- Elles ont ete faconnees par Hibernate en mode "update", qui ne sait ni retirer une
-- contrainte devenue fausse, ni ajouter une colonne NOT NULL a une table deja peuplee --
-- et qui echoue en silence dans le second cas. Ces trois corrections etaient jusqu'ici
-- rejouees a chaque demarrage par du code applicatif ; elles sont desormais a leur place,
-- datees et appliquees une seule fois.
--
-- Sans effet sur une base neuve, ou le schema initial les contient deja.
--

--changeset palettier:002-retirer-contrainte-reference runOnChange:false
--comment La reference d'un tube servait de cle, jusqu'a ce que des gammes sans reference publiee arrivent au catalogue.
ALTER TABLE IF EXISTS oil_paint DROP CONSTRAINT IF EXISTS uk_paint_brand_code;

--changeset palettier:003-colonne-kind runOnChange:false
--comment Nature d'une couche : marche du degrade, ou variation locale. Les couches anterieures appartiennent toutes a l'echelle.
ALTER TABLE IF EXISTS project_layer ADD COLUMN IF NOT EXISTS kind VARCHAR(10) DEFAULT 'LADDER' NOT NULL;

--changeset palettier:004-colonnes-provenance runOnChange:false
--comment Un tube saisi par le peintre ne doit jamais etre efface par le menage du catalogue, et la provenance de ses pigments doit se voir.
ALTER TABLE IF EXISTS oil_paint ADD COLUMN IF NOT EXISTS user_added BOOLEAN DEFAULT FALSE NOT NULL;
ALTER TABLE IF EXISTS oil_paint ADD COLUMN IF NOT EXISTS pigments_verified BOOLEAN DEFAULT FALSE NOT NULL;
