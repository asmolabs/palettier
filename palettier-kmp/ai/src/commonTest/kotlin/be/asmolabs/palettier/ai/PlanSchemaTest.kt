package be.asmolabs.palettier.ai

import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * Les trois facons de dire la meme contrainte.
 *
 * <p>Un schema mal taille n'echoue pas doucement : le moteur rejette la demande entiere,
 * ou repond du texte libre. Ces vues meritent donc d'etre verifiees hors ligne.</p>
 */
class PlanSchemaTest {

    private val zone get() = PlanSchema.plan
        .jsonObject["properties"]!!.jsonObject["zones"]!!.jsonObject["items"]!!.jsonObject

    @Test
    fun `le schema de base impose le format hexadecimal`() {
        val hex = zone["properties"]!!.jsonObject["base"]!!.jsonObject["properties"]!!
            .jsonObject["hex"]!!.jsonObject
        assertEquals("^#[0-9A-Fa-f]{6}$", hex["pattern"]!!.jsonPrimitive.content)
    }

    @Test
    fun `la vue stricte declare tout obligatoire et ferme les objets`() {
        walk(PlanSchema.strict) { node ->
            val properties = node["properties"]?.jsonObject ?: return@walk
            assertEquals(false, node["additionalProperties"]!!.jsonPrimitive.content.toBoolean())
            val required = node["required"]!!.jsonArray.map { it.jsonPrimitive.content }.toSet()
            assertEquals(properties.keys, required, "tout champ doit etre declare obligatoire")
        }
    }

    @Test
    fun `un champ facultatif devient nullable plutot qu'absent`() {
        val zones = PlanSchema.strict["properties"]!!.jsonObject["zones"]!!.jsonObject
        val properties = zones["items"]!!.jsonObject["properties"]!!.jsonObject

        // accent1 etait facultatif : il reste declare, mais admet le nul.
        val accent = properties["accent1"]!!.jsonObject["type"]!!.jsonArray.map { it.jsonPrimitive.content }
        assertEquals(listOf("object", "null"), accent)

        // base etait obligatoire : son type ne bouge pas.
        assertEquals("object", properties["base"]!!.jsonObject["type"]!!.jsonPrimitive.content)
    }

    @Test
    fun `une enumeration nullable admet le nul parmi ses valeurs`() {
        val technique = PlanSchema.strict["properties"]!!.jsonObject["zones"]!!.jsonObject["items"]!!
            .jsonObject["properties"]!!.jsonObject["base"]!!.jsonObject["properties"]!!
            .jsonObject["technique"]!!.jsonObject
        assertContains(technique["enum"]!!.jsonArray, JsonNull)
        assertContains(technique["type"]!!.jsonArray.map { it.jsonPrimitive.content }, "null")
    }

    @Test
    fun `la vue Gemini ne porte ni motif ni champ supplementaire`() {
        walk(PlanSchema.gemini) { node ->
            assertFalse(node.containsKey("pattern"), "Gemini rejette les motifs")
            assertFalse(node.containsKey("additionalProperties"))
        }
        // Ce qu'elle garde : la structure, les enumerations et les bornes de zones.
        val zones = PlanSchema.gemini["properties"]!!.jsonObject["zones"]!!.jsonObject
        assertEquals(6, zones["maxItems"]!!.jsonPrimitive.content.toInt())
        assertTrue(zones["items"]!!.jsonObject["required"]!!.jsonArray.isNotEmpty())
    }

    private fun walk(node: JsonObject, check: (JsonObject) -> Unit) {
        check(node)
        node.values.forEach { value ->
            when (value) {
                is JsonObject -> walk(value, check)
                is JsonArray -> value.filterIsInstance<JsonObject>().forEach { walk(it, check) }
                is JsonPrimitive -> Unit
            }
        }
    }
}
