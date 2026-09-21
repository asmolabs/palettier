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
 * Un schema JSON, retaille a la mesure de chaque moteur.
 *
 * <p>Spring AI absorbait ces differences ; en les portant a la main, on les voit. Ce ne
 * sont pas trois enrobages du meme champ : le mode strict d'OpenAI exige que rien ne soit
 * facultatif, et Gemini rejette la demande entiere s'il rencontre un motif. Une
 * divergence ici ne provoque pas d'erreur -- seulement une reponse en texte libre.</p>
 *
 * <p>Les vues sont derivees et non recopiees : un schema ecrit une fois vaut pour les
 * trois moteurs.</p>
 */
object Schemas {

    /**
     * Le schema au format strict d'OpenAI.
     *
     * <p>OpenAI ne garantit la conformite qu'a deux conditions : aucun champ imprevu, et
     * tous les champs declares obligatoires. Un champ facultatif se dit donc autrement,
     * en admettant la valeur nulle -- ce qui revient au meme pour nous, nos objets lisant
     * l'absence et le nul de la meme facon.</p>
     */
    fun strict(schema: JsonObject): JsonObject = strictly(schema) as JsonObject

    /**
     * Le schema au sous-ensemble OpenAPI que comprend Gemini.
     *
     * <p>Gemini ignore les motifs et les champs supplementaires. Les lui envoyer ferait
     * rejeter la demande entiere, alors on les retire : le format redescend au rang de
     * consigne, et le prompt le repete deja.</p>
     */
    fun openApi(schema: JsonObject): JsonObject = withoutPatterns(schema) as JsonObject

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
