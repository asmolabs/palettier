package be.asmolabs.palettier.ui

import be.asmolabs.palettier.domain.color.Rgb
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * La derniere teinte relevee, portee d'un ecran a l'autre.
 *
 * <p>C'est le geste le plus frequent de l'application : on releve une couleur sur une
 * photo, puis on veut savoir quel tube s'en approche, ou quel melange la donne. Sans ce
 * relais, il faut recopier six caracteres hexadecimaux a la main entre deux ecrans.</p>
 *
 * <p>Un seul objet partage, et aucun ecran ne connait les autres : la pipette ecrit, le
 * catalogue et le melangeur lisent, l'etabli s'en sert pour nommer un melange pose sur la
 * palette. Ajouter un lecteur ne demande de toucher a aucun ecrivain.</p>
 *
 * <p>Les lecteurs ne s'en saisissent pas d'autorite : ils la proposent. Une teinte relevee
 * il y a une heure ne doit pas ecraser celle qu'on vient de taper -- c'est le peintre qui
 * decide quand la reprendre.</p>
 */
class SampledColour {

    private val _colour = MutableStateFlow<Rgb?>(null)

    val colour: StateFlow<Rgb?> = _colour.asStateFlow()

    fun set(colour: Rgb?) {
        _colour.value = colour
    }
}
