package be.asmolabs.palettier.core.backup;

import java.nio.file.Path;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfSystemProperty;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.test.context.TestPropertySource;

/**
 * Ecrit une sauvegarde depuis une vraie base H2, pour la relire depuis l'application Kotlin.
 *
 * <p>A lancer a la main quand on veut faire passer les donnees du poste d'une application a
 * l'autre. Ce n'est pas un test : il ne s'active que si on lui donne une destination, ce qui
 * le laisse muet dans la suite.</p>
 *
 * <pre>
 * mvn -pl palettier-core test -Dtest=RealBackupExporter -DfailIfNoTests=false \
 *     -Dpalettier.export.db=/chemin/vers/db \
 *     -Dpalettier.export.to=/chemin/vers/sauvegarde.zip
 * </pre>
 *
 * <p>Le chemin de la base est donne sans le suffixe {@code .mv.db}, comme H2 l'attend.
 * Travailler sur une copie evite de toucher a la base de l'atelier.</p>
 */
@SpringBootTest
@EnabledIfSystemProperty(named = "palettier.export.to", matches = ".+")
@TestPropertySource(properties = {
        "spring.datasource.url=jdbc:h2:file:${palettier.export.db};AUTO_SERVER=FALSE",
        "spring.datasource.username=sa",
        "spring.datasource.password="
})
class RealBackupExporter {

    @Autowired
    private BackupService backup;

    @Test
    void writeArchive() throws Exception {
        Path destination = Path.of(System.getProperty("palettier.export.to"));
        BackupService.Summary summary = backup.export(destination);
        System.out.printf(
                "SAUVEGARDE %s : %d tubes, %d palettes, %d projets, %d recettes, %d photos, %d ko%n",
                destination, summary.paints(), summary.palettes(), summary.projects(),
                summary.recipes(), summary.photos(), summary.bytes() / 1024);
    }
}
