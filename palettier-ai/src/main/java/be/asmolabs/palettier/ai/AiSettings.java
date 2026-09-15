package be.asmolabs.palettier.ai;

import java.util.Optional;
import org.springframework.stereotype.Component;

/**
 * Reglages de l'assistant valables pour la session en cours.
 *
 * <p>Le modele choisi ici prime sur celui du fichier de configuration, sans redemarrage.
 * C'est volontairement une preference de session et non un fichier : changer de modele
 * pour comparer deux resultats est un geste d'essai, pas une decision a graver.</p>
 */
@Component
public class AiSettings {

    private volatile String model;

    /** Modele retenu pour la session, ou vide pour celui de la configuration. */
    public Optional<String> model() {
        return Optional.ofNullable(model).filter(name -> !name.isBlank());
    }

    public void useModel(String model) {
        this.model = model == null || model.isBlank() ? null : model.trim();
    }
}
