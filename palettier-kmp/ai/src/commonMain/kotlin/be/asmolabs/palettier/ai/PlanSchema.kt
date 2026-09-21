package be.asmolabs.palettier.ai

import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
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

    /**
     * Le meme schema, au format strict d'OpenAI.
     *
     * <p>OpenAI ne garantit la conformite qu'a deux conditions : aucun champ imprevu, et
     * tous les champs declares obligatoires. Un champ facultatif se dit donc autrement,
     * en admettant la valeur nulle -- ce qui revient au meme pour nous, {@link PlanDraft}
     * lisant l'absence et le nul de la meme facon.</p>
     */
    val strict: JsonObject = strictly(plan) as JsonObject

    /**
     * Le meme schema, au sous-ensemble OpenAPI que comprend Gemini.
     *
     * <p>Gemini ignore les motifs et les champs supplementaires. Les lui envoyer ferait
     * rejeter la demande entiere, alors on les retire : le format hexadecimal redescend
     * au rang de consigne, et le prompt le repete deja.</p>
     */
    val gemini: JsonObject = withoutPatterns(plan) as JsonObject

    private fun strictly(node: JsonElement): JsonElement = when {
        node is JsonArray -> JsonArray(node.map { strictly(it) })
        node !is JsonObject -> node
        node["properties"] is JsonObject -> {
            val properties = node["properties"] as JsonObject
            val optional = properties.keys - requiredOf(node)
            buildJsonObject {
                node.forEach { (key, value) ->
                    if (key != "properties" && key != "required") put(key, strictly(value))
                }
                putJsonObject("properties") {
                    properties.forEach { (name, value) ->
                        val done = strictly(value)
                        put(name, if (name in optional) nullable(done) else done)
                    }
                }
                putJsonArray("required") { properties.keys.forEach { add(JsonPrimitive(it)) } }
                put("additionalProperties", false)
            }
        }
        else -> JsonObject(node.mapValues { (_, value) -> strictly(value) })
    }

    /** Un champ facultatif, dit a la maniere d'OpenAI : present, mais admettant le nul. */
    private fun nullable(node: JsonElement): JsonElement {
        if (node !is JsonObject) return node
        return buildJsonObject {
            node.forEach { (key, value) ->
                when (key) {
                    "type" -> put("type", buildJsonArray { add(value); add(JsonPrimitive("null")) })
                    "enum" -> put("enum", buildJsonArray {
                        (value as JsonArray).forEach { add(it) }
                        add(JsonNull)
                    })
                    else -> put(key, value)
                }
            }
        }
    }

    private fun withoutPatterns(node: JsonElement): JsonElement = when (node) {
        is JsonArray -> JsonArray(node.map { withoutPatterns(it) })
        is JsonObject -> JsonObject(
            node.filterKeys { it != "pattern" && it != "additionalProperties" }
                .mapValues { (_, value) -> withoutPatterns(value) }
        )
        else -> node
    }

    private fun requiredOf(node: JsonObject): Set<String> =
        (node["required"] as? JsonArray)?.map { (it as JsonPrimitive).content }?.toSet() ?: emptySet()
}
