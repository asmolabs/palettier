package be.asmolabs.palettier.ai;

import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

/**
 * Porte d'entree du module assistant, a importer depuis l'application hote.
 *
 * <p>Le module est inerte tant qu'aucun moteur n'est selectionne par
 * {@code spring.ai.model.chat}. Aucune cle, aucun appel reseau, aucun changement de
 * comportement : c'est la condition pour que l'assistant reste un supplement.</p>
 */
@Configuration(proxyBeanMethods = false)
@ComponentScan("be.asmolabs.palettier.ai")
public class AiConfiguration {
}
