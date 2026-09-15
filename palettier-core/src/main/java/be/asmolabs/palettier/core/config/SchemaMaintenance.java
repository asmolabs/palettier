package be.asmolabs.palettier.core.config;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.ApplicationArguments;
import org.springframework.boot.ApplicationRunner;
import org.springframework.core.annotation.Order;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.stereotype.Component;

/**
 * Retire les contraintes devenues fausses sur les bases deja en service.
 *
 * <p>Hibernate en mode {@code update} sait ajouter une colonne ou une table, mais jamais
 * supprimer une contrainte existante. Un index d'unicite pose par une version anterieure
 * survit donc a sa propre disparition du modele, et bloque des insertions parfaitement
 * legitimes. C'est ce qui s'est produit avec {@code uk_paint_brand_code} : la reference
 * d'un tube servait de cle, jusqu'a ce que des gammes sans reference publiee arrivent au
 * catalogue.</p>
 *
 * <p>C'est un correctif cible, pas une solution durable. Des que le schema se stabilisera,
 * la bonne reponse sera un outil de migration versionnee (Flyway ou Liquibase) et
 * {@code ddl-auto: validate}.</p>
 */
@Component
@Order(0)
class SchemaMaintenance implements ApplicationRunner {

    private static final Logger log = LoggerFactory.getLogger(SchemaMaintenance.class);

    /**
     * Reparations du schema, jouees a chaque demarrage et sans effet quand il n'y a rien
     * a faire.
     *
     * <p>Deux familles : les contraintes d'hier qu'Hibernate ne sait pas retirer, et les
     * colonnes qu'il n'a pas su ajouter. Une colonne NOT NULL ajoutee a une table deja
     * peuplee echoue silencieusement en mode {@code update} : la base continue de tourner
     * sans elle jusqu'a la premiere lecture, qui casse.</p>
     */
    private static final String[] REPAIRS = {
            "alter table if exists oil_paint drop constraint if exists uk_paint_brand_code",
            "alter table if exists project_layer add column if not exists kind varchar(10) default 'LADDER' not null"
    };

    private final JdbcTemplate jdbc;

    SchemaMaintenance(JdbcTemplate jdbc) {
        this.jdbc = jdbc;
    }

    @Override
    public void run(ApplicationArguments args) {
        for (String statement : REPAIRS) {
            try {
                jdbc.execute(statement);
            } catch (RuntimeException e) {
                // Rien a nettoyer sur une base neuve : ce n'est pas une erreur.
                log.debug("Nettoyage de schema sans effet : {}", statement, e);
            }
        }
    }
}
