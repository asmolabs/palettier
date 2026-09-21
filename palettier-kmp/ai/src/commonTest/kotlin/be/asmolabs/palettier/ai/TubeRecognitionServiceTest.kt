package be.asmolabs.palettier.ai

import be.asmolabs.palettier.domain.image.PixelMap
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.paint.PaintMatcher
import be.asmolabs.palettier.image.ImageDecoder
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * La lecture d'etiquettes.
 *
 * <p>Le modele ne decide pas ce que le peintre possede : il propose des libelles, que
 * l'application rapproche du catalogue avec une confiance. Ce partage est ce qui se
 * verifie ici, et il tient hors ligne.</p>
 */
class TubeRecognitionServiceTest {

    private class Reading(private val answer: String) : ChatEngine {
        override val name = "lecteur"
        var last: JsonRequest? = null
        override suspend fun ask(request: JsonRequest): String {
            last = request
            return answer
        }
    }

    private object Plain : ImageDecoder {
        var scaled = 0
        override suspend fun decode(bytes: ByteArray) = PixelMap(4, 4, IntArray(16))
        override suspend fun scaleTo(bytes: ByteArray, maxEdge: Int): ByteArray {
            scaled++
            return bytes
        }
    }

    private val catalogue = listOf(
        Paint(id = 1, brand = "Winsor & Newton", name = "Burnt Umber", hexColor = "#8A5A44"),
        Paint(id = 2, brand = "Gamblin", name = "Ivory Black", hexColor = "#221F1C"),
        Paint(id = 3, brand = "Abteilung 502", name = "Shadow Brown", hexColor = "#4A3A2E"),
    )

    private fun service(engine: ChatEngine?) =
        TubeRecognitionService(ChatEngines { engine }, PaintMatcher(), Plain)

    @Test
    fun `les libelles lus sont rapproches du catalogue`() = runTest {
        val read = service(Reading("""{"tubes":["Winsor & Newton Burnt Umber 37ml","Abteilung 502 Shadow Brown"]}"""))
            .identify(byteArrayOf(1), catalogue)

        assertEquals(2, read.size)
        assertEquals("Winsor & Newton Burnt Umber 37ml", read.first().label)
        assertEquals(1L, read.first().match?.paint?.id)
        assertEquals(3L, read.last().match?.paint?.id)
    }

    @Test
    fun `un libelle sans correspondance reste propose, sans rapprochement`() = runTest {
        val read = service(Reading("""{"tubes":["Marque inconnue Bleu de nulle part"]}"""))
            .identify(byteArrayOf(1), catalogue)

        assertEquals(1, read.size)
        assertNull(read.single().match, "mieux vaut aucun rapprochement qu'un mauvais")
    }

    @Test
    fun `les libelles vides sont ecartes`() = runTest {
        val read = service(Reading("""{"tubes":["  ","Gamblin Ivory Black",""]}"""))
            .identify(byteArrayOf(1), catalogue)

        assertEquals(1, read.size)
        assertEquals("Gamblin Ivory Black", read.single().label)
    }

    @Test
    fun `la photo est reduite, et le schema part avec la demande`() = runTest {
        val engine = Reading("""{"tubes":[]}""")
        val before = Plain.scaled
        service(engine).identify(byteArrayOf(1, 2, 3), catalogue)

        assertEquals(before + 1, Plain.scaled)
        val request = assertNotNull(engine.last)
        assertEquals(1, request.photos.size)
        assertTrue("tubes" in request.schema["properties"].toString())
        // Une liste d'etiquettes tient dans peu de place : inutile de reserver large.
        assertEquals(TubeRecognitionService.MAX_TOKENS, request.maxTokens)
    }

    @Test
    fun `sans moteur, la lecture se refuse plutot que d'echouer plus loin`() = runTest {
        val service = service(null)
        assertEquals(false, service.isAvailable())
        val refus = assertFailsWith<PlanUnavailable> { service.identify(byteArrayOf(1), catalogue) }
        assertContains(refus.message!!, "parametres")
    }

    @Test
    fun `une reponse qui n'est pas une liste donne un message exploitable`() = runTest {
        val refus = assertFailsWith<PlanUnavailable> {
            service(Reading("je vois trois tubes")).identify(byteArrayOf(1), catalogue)
        }
        assertContains(refus.message!!, "etiquettes sont lisibles")
    }
}
