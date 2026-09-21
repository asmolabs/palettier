package be.asmolabs.palettier.ai

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.engine.mock.respondError
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import io.ktor.http.HttpHeaders
import io.ktor.utils.io.ByteReadChannel
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/** Ce que le moteur envoie, et ce qu'il fait de ce qu'il recoit. */
class OllamaEngineTest {

    private var lastBody: String = ""

    /** Un moteur branche sur un serveur simule, qui retient la requete envoyee. */
    private fun okWith(content: String) = OllamaEngine(
        HttpClient(MockEngine { request ->
            lastBody = (request.body as io.ktor.http.content.TextContent).text
            respond(
                ByteReadChannel(
                    Json.encodeToString(
                        buildJsonObject {
                            putJsonObject("message") { put("role", "assistant"); put("content", content) }
                        }
                    )
                ),
                HttpStatusCode.OK,
                headersOf(HttpHeaders.ContentType, "application/json"),
            )
        })
    )

    private fun failing(status: HttpStatusCode) = OllamaEngine(
        HttpClient(MockEngine { respondError(status) })
    )

    private val plan = """
        {"approach":"Palette courte.","zones":[{"name":"Visage","material":"Peau","note":"",
        "base":{"hex":"#C98F72","technique":"Glacis","note":""},
        "shadow1":{"hex":"#8A5F4A","technique":"Glacis","note":""},
        "shadow2":{"hex":"#5A3B2E","technique":"Glacis","note":""},
        "highlight1":{"hex":"#E0B49A","technique":"Glacis","note":""},
        "highlight2":{"hex":"#F2D8C4","technique":"Glacis","note":""}}]}
    """.trimIndent()

    @Test
    fun `le schema part avec la demande, pas une consigne dans le texte`() = runTest {
        okWith(plan).draft(PlanRequest("systeme", "utilisateur"))

        val sent = Json.parseToJsonElement(lastBody).jsonObject
        // C'est la contrainte qui rend le JSON invalide impossible : elle doit etre la.
        val format = sent["format"]!!.jsonObject
        assertEquals("object", format["type"]!!.jsonPrimitive.content)
        assertTrue("zones" in format["properties"]!!.jsonObject)

        // Et la fenetre de contexte est posee explicitement : la valeur par defaut
        // d'Ollama coupe la reponse en plein plan.
        val options = sent["options"]!!.jsonObject
        assertEquals(32768, options["num_ctx"]!!.jsonPrimitive.content.toInt())
        assertEquals(16384, options["num_predict"]!!.jsonPrimitive.content.toInt())
    }

    @Test
    fun `une reponse conforme devient un plan`() = runTest {
        val draft = okWith(plan).draft(PlanRequest("systeme", "utilisateur"))

        assertEquals("Palette courte.", draft.approach)
        assertEquals(1, draft.zones.size)
        assertEquals("#C98F72", draft.zones.single().base?.hex)
        assertEquals("#5A3B2E", draft.zones.single().shadow2?.hex)
    }

    @Test
    fun `les photos partent en base64, et sont annoncees`() = runTest {
        okWith(plan).draft(
            PlanRequest("systeme", "utilisateur", listOf(PhotoInput(byteArrayOf(1, 2, 3), "la piece")))
        )

        val messages = Json.parseToJsonElement(lastBody).jsonObject["messages"]!!.jsonArray
        val images = messages.last().jsonObject["images"]!!.jsonArray
        assertEquals(1, images.size)
        assertEquals("AQID", images.first().jsonPrimitive.content)
    }

    @Test
    fun `une reponse illisible donne un message exploitable, pas une erreur technique`() = runTest {
        val failure = assertFailsWith<PlanUnavailable> {
            okWith("{\"approach\": \"coupe en plein").draft(PlanRequest("s", "u"))
        }
        assertTrue("plan exploitable" in failure.message!!)
    }

    @Test
    fun `un refus du serveur est signale tel quel`() = runTest {
        val failure = assertFailsWith<PlanUnavailable> {
            failing(HttpStatusCode.NotFound).draft(PlanRequest("s", "u"))
        }
        assertTrue("404" in failure.message!!)
    }
}
