package be.asmolabs.palettier.ai

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.engine.mock.respondError
import io.ktor.client.request.HttpRequestData
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import io.ktor.utils.io.ByteReadChannel
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/**
 * Les deux moteurs distants : ce qu'ils envoient, et ce qu'ils font d'une reponse.
 *
 * <p>Chacun dit la contrainte a sa facon -- {@code response_format} chez OpenAI, {@code
 * responseSchema} chez Gemini -- et c'est precisement ce qui merite d'etre verifie : une
 * cle mal placee ne provoque pas d'erreur, seulement une reponse en texte libre.</p>
 */
class RemoteEnginesTest {

    private var lastBody: String = ""
    private var lastRequest: HttpRequestData? = null

    private fun client(reply: String) = HttpClient(MockEngine { request ->
        lastRequest = request
        lastBody = (request.body as TextContent).text
        respond(
            ByteReadChannel(reply),
            HttpStatusCode.OK,
            headersOf(HttpHeaders.ContentType, "application/json"),
        )
    })

    private fun refusing(status: HttpStatusCode) = HttpClient(MockEngine { respondError(status) })

    private val plan = """
        {"approach":"Palette courte.","zones":[{"name":"Visage","material":"Peau","note":"",
        "base":{"hex":"#C98F72","technique":"Glacis","note":""},
        "shadow1":{"hex":"#8A5F4A","technique":"Glacis","note":""},
        "shadow2":{"hex":"#5A3B2E","technique":"Glacis","note":""},
        "highlight1":{"hex":"#E0B49A","technique":"Glacis","note":""},
        "highlight2":{"hex":"#F2D8C4","technique":"Glacis","note":""}}]}
    """.trimIndent()

    private fun openAiReply(content: String) =
        Json.encodeToString(
            mapOf("choices" to listOf(mapOf("message" to mapOf("content" to content))))
        )

    private fun geminiReply(content: String) =
        Json.encodeToString(
            mapOf("candidates" to listOf(mapOf("content" to mapOf("parts" to listOf(mapOf("text" to content))))))
        )

    // --- OpenAI --------------------------------------------------------------

    @Test
    fun `OpenAI recoit le schema strict, et la cle en entete`() = runTest {
        OpenAiEngine(client(openAiReply(plan)), "cle-secrete")
            .draft(PlanRequest("systeme", "utilisateur"))

        val sent = Json.parseToJsonElement(lastBody).jsonObject
        val format = sent["response_format"]!!.jsonObject
        assertEquals("json_schema", format["type"]!!.jsonPrimitive.content)

        val schema = format["json_schema"]!!.jsonObject
        assertEquals("true", schema["strict"]!!.jsonPrimitive.content)
        // Le mode strict n'a de valeur que si le schema est effectivement ferme.
        assertEquals(
            "false",
            schema["schema"]!!.jsonObject["additionalProperties"]!!.jsonPrimitive.content,
        )

        assertEquals("Bearer cle-secrete", lastRequest!!.headers[HttpHeaders.Authorization])
        assertTrue(lastRequest!!.url.toString().endsWith("/chat/completions"))
    }

    @Test
    fun `OpenAI recoit les photos en URL de donnees`() = runTest {
        OpenAiEngine(client(openAiReply(plan)), "cle")
            .draft(PlanRequest("s", "u", listOf(PhotoInput(byteArrayOf(1, 2, 3), "la piece"))))

        val content = Json.parseToJsonElement(lastBody).jsonObject["messages"]!!
            .jsonArray.last().jsonObject["content"]!!.jsonArray
        assertEquals("text", content.first().jsonObject["type"]!!.jsonPrimitive.content)
        assertEquals(
            "data:image/jpeg;base64,AQID",
            content.last().jsonObject["image_url"]!!.jsonObject["url"]!!.jsonPrimitive.content,
        )
    }

    @Test
    fun `une reponse conforme d'OpenAI devient un plan`() = runTest {
        val draft = OpenAiEngine(client(openAiReply(plan)), "cle").draft(PlanRequest("s", "u"))
        assertEquals("Palette courte.", draft.approach)
        assertEquals("#C98F72", draft.zones.single().base?.hex)
    }

    @Test
    fun `un refus d'OpenAI est signale tel quel`() = runTest {
        val failure = assertFailsWith<PlanUnavailable> {
            OpenAiEngine(refusing(HttpStatusCode.Unauthorized), "mauvaise cle").draft(PlanRequest("s", "u"))
        }
        assertTrue("401" in failure.message!!)
    }

    // --- Gemini --------------------------------------------------------------

    @Test
    fun `Gemini recoit un schema sans motif, et le modele dans l'URL`() = runTest {
        GeminiEngine(client(geminiReply(plan)), "cle", defaultModel = "gemini-2.5-pro")
            .draft(PlanRequest("systeme", "utilisateur"))

        val config = Json.parseToJsonElement(lastBody).jsonObject["generationConfig"]!!.jsonObject
        assertEquals("application/json", config["responseMimeType"]!!.jsonPrimitive.content)
        assertTrue("pattern" !in config["responseSchema"].toString(), "Gemini rejette les motifs")

        // La consigne de systeme est un champ a part, et non un message de plus.
        val system = Json.parseToJsonElement(lastBody).jsonObject["systemInstruction"]!!.jsonObject
        assertEquals("systeme", system["parts"]!!.jsonArray.single().jsonObject["text"]!!.jsonPrimitive.content)

        assertTrue(lastRequest!!.url.toString().endsWith("/models/gemini-2.5-pro:generateContent"))
        assertEquals("cle", lastRequest!!.headers["x-goog-api-key"])
    }

    @Test
    fun `le modele demande pour une seule question l'emporte sur celui du moteur`() = runTest {
        GeminiEngine(client(geminiReply(plan)), "cle", defaultModel = "gemini-2.5-pro")
            .draft(PlanRequest("s", "u", model = "gemini-2.5-flash"))

        assertTrue(lastRequest!!.url.toString().contains("gemini-2.5-flash"))
    }

    @Test
    fun `Gemini recoit les photos en pieces jointes`() = runTest {
        GeminiEngine(client(geminiReply(plan)), "cle")
            .draft(PlanRequest("s", "u", listOf(PhotoInput(byteArrayOf(1, 2, 3), "la piece"))))

        val parts = Json.parseToJsonElement(lastBody).jsonObject["contents"]!!
            .jsonArray.single().jsonObject["parts"]!!.jsonArray
        val data = parts.last().jsonObject["inline_data"]!!.jsonObject
        assertEquals("image/jpeg", data["mime_type"]!!.jsonPrimitive.content)
        assertEquals("AQID", data["data"]!!.jsonPrimitive.content)
    }

    @Test
    fun `une reponse de Gemini decoupee en morceaux est recollee`() = runTest {
        val reply = Json.encodeToString(
            mapOf(
                "candidates" to listOf(
                    mapOf(
                        "content" to mapOf(
                            "parts" to listOf(
                                mapOf("text" to plan.substring(0, 20)),
                                mapOf("text" to plan.substring(20)),
                            )
                        )
                    )
                )
            )
        )
        val draft = GeminiEngine(client(reply), "cle").draft(PlanRequest("s", "u"))
        assertEquals("Palette courte.", draft.approach)
    }

    @Test
    fun `une reponse illisible donne le meme message pour les trois moteurs`() = runTest {
        val openAi = assertFailsWith<PlanUnavailable> {
            OpenAiEngine(client(openAiReply("{\"approach\": \"coupe en plein")), "cle").draft(PlanRequest("s", "u"))
        }
        val gemini = assertFailsWith<PlanUnavailable> {
            GeminiEngine(client(geminiReply("pas du JSON du tout")), "cle").draft(PlanRequest("s", "u"))
        }
        assertTrue("plan exploitable" in openAi.message!!)
        assertEquals(openAi.message, gemini.message)
    }
}
