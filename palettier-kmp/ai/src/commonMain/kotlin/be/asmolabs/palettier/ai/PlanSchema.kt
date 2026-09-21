package be.asmolabs.palettier.ai

import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject

/**
 * Le schema JSON de la reponse attendue, ecrit a la main.
 *
 * <p>Spring AI le derivait de la classe. Ici il n'y a pas d'equivalent, et c'est en
 * definitive une bonne chose : ce schema n'est pas une consigne ajoutee au prompt mais
 * une contrainte remise au moteur, qui lui interdit de produire le jeton qui casserait le
 * JSON. Il merite donc d'etre lu, pas devine.</p>
 *
 * <p>Le format hexadecimal est impose par un motif. Un modele qui repondrait "brun chaud"
 * ne le peut simplement plus.</p>
 */
object PlanSchema {

    private const val HEX_PATTERN = "^#[0-9A-Fa-f]{6}$"

    /** Les techniques admises, mot pour mot : le nom libre invitait a l'approximation. */
    private val TECHNIQUES = listOf(
        "Aplat de base", "Jus a l'huile", "Jus capillaire (pin wash)", "Filtre", "Glacis",
        "Fondu / degrade", "Dot fading", "Coulures et salissures",
        "Oil Paint Rendering (OPR)", "Eclaircis et points lumineux",
    )

    private fun layer() = buildJsonObject {
        put("type", "object")
        putJsonObject("properties") {
            putJsonObject("hex") {
                put("type", "string")
                put("pattern", HEX_PATTERN)
            }
            putJsonObject("technique") {
                put("type", "string")
                putJsonArray("enum") { TECHNIQUES.forEach { add(JsonPrimitive(it)) } }
            }
            putJsonObject("note") { put("type", "string") }
        }
        putJsonArray("required") { add(JsonPrimitive("hex")) }
    }

    private fun accent() = buildJsonObject {
        put("type", "object")
        putJsonObject("properties") {
            putJsonObject("name") { put("type", "string") }
            putJsonObject("hex") {
                put("type", "string")
                put("pattern", HEX_PATTERN)
            }
            putJsonObject("technique") {
                put("type", "string")
                putJsonArray("enum") { TECHNIQUES.forEach { add(JsonPrimitive(it)) } }
            }
            putJsonObject("note") { put("type", "string") }
        }
    }

    /** Le schema complet d'un plan. */
    val plan: JsonObject = buildJsonObject {
        put("type", "object")
        putJsonObject("properties") {
            putJsonObject("approach") { put("type", "string") }
            putJsonObject("zones") {
                put("type", "array")
                put("minItems", 1)
                put("maxItems", 6)
                putJsonObject("items") {
                    put("type", "object")
                    putJsonObject("properties") {
                        putJsonObject("name") { put("type", "string") }
                        putJsonObject("material") { put("type", "string") }
                        putJsonObject("note") { put("type", "string") }
                        put("base", layer())
                        put("shadow1", layer())
                        put("shadow2", layer())
                        put("highlight1", layer())
                        put("highlight2", layer())
                        put("accent1", accent())
                        put("accent2", accent())
                        put("accent3", accent())
                    }
                    putJsonArray("required") {
                        listOf("name", "base", "shadow1", "shadow2", "highlight1", "highlight2")
                            .forEach { add(JsonPrimitive(it)) }
                    }
                }
            }
        }
        putJsonArray("required") {
            add(JsonPrimitive("approach"))
            add(JsonPrimitive("zones"))
        }
    }
}
