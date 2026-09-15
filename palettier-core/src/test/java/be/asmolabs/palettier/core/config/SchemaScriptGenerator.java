package be.asmolabs.palettier.core.config;

import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.test.context.TestPropertySource;

/**
 * Ecrit le schema que le modele attend, pour servir de base aux migrations.
 *
 * <p>A relancer quand on veut comparer le schema d'Hibernate a celui des migrations,
 * par exemple pour rediger une nouvelle version. Desactive en temps normal : ce n'est
 * pas un test, c'est un outil.</p>
 */
@Disabled("Outil : a lancer a la main quand on redige une migration")
@SpringBootTest
@TestPropertySource(properties = {
        "spring.ai.model.chat=none",
        "spring.liquibase.enabled=false",
        "spring.jpa.hibernate.ddl-auto=none",
        "spring.jpa.properties.jakarta.persistence.schema-generation.database.action=create",
        "spring.datasource.url=jdbc:h2:mem:schemagen;DB_CLOSE_DELAY=-1",
        "spring.jpa.properties.jakarta.persistence.schema-generation.scripts.action=create",
        "spring.jpa.properties.jakarta.persistence.schema-generation.scripts.create-target=target/schema-attendu.sql",
        "spring.jpa.properties.jakarta.persistence.schema-generation.create-source=metadata"
})
class SchemaScriptGenerator {

    @Test
    void writeSchema() {
        // Le contexte suffit : Hibernate ecrit le script pendant son demarrage.
    }
}
