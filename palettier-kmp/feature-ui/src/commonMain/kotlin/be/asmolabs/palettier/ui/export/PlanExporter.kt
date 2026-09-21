package be.asmolabs.palettier.ui.export

import be.asmolabs.palettier.domain.plan.PaintingPlan

/**
 * Met un plan de peinture sur papier.
 *
 * <p>Un plan sert a l'etabli, ou l'on ne consulte pas un ecran les mains grasses de
 * diluant. La feuille imprimee doit donc se suffire a elle-meme : pastilles dessinees,
 * dosages ecrits en toutes lettres, ecart mesure indique pour qu'on sache d'avance ou la
 * palette va manquer.</p>
 *
 * <p>L'interface rend des octets et ne connait pas de chemin. Ou ranger un fichier est
 * une question de plateforme -- un dossier choisi sur le bureau, un partage sur un
 * telephone -- et le reste n'en depend pas.</p>
 *
 * <p>Il n'y a pas d'expect/actual ici, volontairement : produire un PDF demande une
 * bibliotheque, et il n'est pas question d'en imposer une a une cible qui n'a pas encore
 * d'application. Un ecran qui n'a pas d'exportateur n'affiche pas le bouton.</p>
 */
interface PlanExporter {

    /**
     * @return le document complet, pret a etre ecrit ou partage
     * @throws IllegalStateException si le document ne peut pas etre produit
     */
    suspend fun render(plan: PaintingPlan): ByteArray
}
