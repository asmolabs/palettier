package be.asmolabs.palettier.ai

import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.paint.PaintMatcher
import be.asmolabs.palettier.image.ImageDecoder
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject
import kotlinx.serialization.json.JsonPrimitive

/** Ce que le modele renvoie : des libelles, rien de plus. */
@Serializable
data class TubeLabels(val tubes: List<String> = emptyList())

/**
 * Un tube lu, et le rapprochement propose.
 *
 * @param match null quand aucune fiche du catalogue ne correspond assez bien
 */
data class Identification(val label: String, val match: PaintMatcher.Match?)

/**
 * Lit les etiquettes d'une photo de tubes pour constituer l'inventaire.
 *
 * <p>Saisir quatre cents cases a la main est decourageant ; photographier sa boite ne
 * coute rien. Le modele ne fait que <em>lire</em> : il ne decide pas ce que vous
 * possedez, il propose des libelles que l'application rapproche ensuite du catalogue,
 * avec une confiance. La derniere main reste au peintre.</p>
 */
class TubeRecognitionService(
    private val engines: ChatEngines,
    private val matcher: PaintMatcher,
    private val decoder: ImageDecoder,
) {

    private val json = Json { ignoreUnknownKeys = true }

    suspend fun isAvailable(): Boolean = engines.current() != null

    /**
     * @param photo     image des tubes, brute ; elle est reduite avant l'envoi
     * @param catalogue fiches parmi lesquelles chercher
     * @param model     modele a employer, ou null pour celui des reglages
     */
    suspend fun identify(photo: ByteArray, catalogue: List<Paint>, model: String? = null): List<Identification> {
        val engine = engines.current() ?: throw PlanUnavailable(
            "Aucun moteur de conversation configure. Choisissez-en un dans les parametres."
        )

        val answer = engine.ask(
            JsonRequest(
                system = SYSTEM_PROMPT,
                user = "Quels tubes vois-tu sur cette photo ?",
                schema = SCHEMA,
                photos = listOf(PhotoInput(decoder.scaleTo(photo, SUBMITTED_EDGE), "les tubes")),
                model = model,
                maxTokens = MAX_TOKENS,
            )
        )

        val read = runCatching { json.decodeFromString<TubeLabels>(answer) }.getOrElse {
            throw PlanUnavailable(
                "Le modele n'a pas su lire cette photo. Verifiez qu'il sait traiter les images, " +
                    "et que les etiquettes sont lisibles.",
                it,
            )
        }

        return read.tubes
            .mapNotNull { it.trim().takeIf(String::isNotBlank) }
            .map { label -> Identification(label, matcher.match(label, catalogue)) }
    }

    companion object {

        /** Une liste d'etiquettes tient dans peu de place : inutile de reserver large. */
        const val MAX_TOKENS = 4096

        /** Assez pour lire une etiquette, pas assez pour payer du transport inutile. */
        const val SUBMITTED_EDGE = 1280

        val SYSTEM_PROMPT = """
            Tu lis les etiquettes de tubes de peinture sur une photo.

            Tu renvoies la liste des tubes visibles, un libelle par tube, en recopiant ce
            qui est ecrit : marque puis nom de la couleur. Par exemple
            "Winsor & Newton Burnt Umber" ou "Abteilung 502 Shadow Brown".

            Ne devine pas ce que tu ne lis pas. Un tube dont l'etiquette est masquee, floue
            ou de dos ne doit pas figurer dans la liste : mieux vaut en oublier un que
            d'en inventer un. N'ajoute ni volume, ni serie, ni commentaire.
        """.trimIndent()

        /** Le schema, le meme pour les trois moteurs : chacun le retaille a sa mesure. */
        val SCHEMA: JsonObject = buildJsonObject {
            put("type", "object")
            putJsonObject("properties") {
                putJsonObject("tubes") {
                    put("type", "array")
                    putJsonObject("items") { put("type", "string") }
                }
            }
            putJsonArray("required") { add(JsonPrimitive("tubes")) }
        }
    }
}
