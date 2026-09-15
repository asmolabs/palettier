--liquibase formatted sql
--
-- Reference precedente d'un tube renumerote par son fabricant.
--
-- Abteilung 502 a renumerote toute sa gamme : ABT004 est devenu AKABT004, et quinze
-- couleurs ont disparu au passage. Le peintre garde sur son etagere des tubes des deux
-- epoques, souvent melanges. Sans cette colonne, un tube ancien reste introuvable.
--
-- Colonne facultative et vide partout ailleurs : les autres gammes n'ont pas change de
-- numerotation.
--

--changeset palettier:005-colonne-reference-ancienne runOnChange:false
--comment Reference d'avant la renumerotation, pour retrouver un tube achete sous son ancienne etiquette.
ALTER TABLE IF EXISTS oil_paint ADD COLUMN IF NOT EXISTS legacy_code VARCHAR(40) DEFAULT '' NOT NULL;
