package be.asmolabs.palettier.ai;

/**
 * Une photo soumise a l'assistant.
 *
 * <p>Ces images servent au jugement, jamais a la mesure : une photo est compressee,
 * redimensionnee, et prise sous une lumiere quelconque. Les couleurs de l'application
 * viennent de la pipette et du catalogue, pas de ce que le modele croit voir ici.</p>
 *
 * @param data    contenu brut du fichier image ; la reduction au format d'envoi est
 *                faite par le service, pas par l'appelant
 * @param caption ce que l'image represente, dit au modele en toutes lettres
 */
public record PhotoInput(byte[] data, String caption) {

    public static final String MIME_TYPE = "image/jpeg";
}
