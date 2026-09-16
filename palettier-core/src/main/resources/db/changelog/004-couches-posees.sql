--liquibase formatted sql
--
-- Ce qui a deja ete peint, et dans quelles conditions.
--
-- Jusqu'ici un projet ne disait que ce qu'il fallait faire, jamais ce qui etait fait.
-- Les durees de sechage restaient donc theoriques : le logiciel savait annoncer "trois
-- jours", jamais "cette cape est recouvrable depuis hier soir".
--
-- Les conditions de l'atelier et la vitesse de sechage du melange sont figees avec la
-- pose plutot que relues a l'affichage. L'huile a seche avec la temperature du jour ou
-- elle a ete posee ; recalculer avec celle d'aujourd'hui, ou avec une palette remaniee
-- depuis, reecrirait le passe.
--
-- Colonnes toutes facultatives : une couche jamais peinte les laisse vides, et les
-- projets anterieurs restent lisibles tels quels.
--

--changeset palettier:006-couches-posees runOnChange:false
--comment Date de pose d'une couche et conditions figees avec elle, pour savoir quand la piece est reprenable.
ALTER TABLE IF EXISTS project_layer ADD COLUMN IF NOT EXISTS applied_at TIMESTAMP(6) WITH TIME ZONE;
ALTER TABLE IF EXISTS project_layer ADD COLUMN IF NOT EXISTS applied_temperature FLOAT(53);
ALTER TABLE IF EXISTS project_layer ADD COLUMN IF NOT EXISTS applied_humidity FLOAT(53);
ALTER TABLE IF EXISTS project_layer ADD COLUMN IF NOT EXISTS applied_ventilation VARCHAR(10);
ALTER TABLE IF EXISTS project_layer ADD COLUMN IF NOT EXISTS applied_drying_class VARCHAR(10);
