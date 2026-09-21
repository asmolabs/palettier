package be.asmolabs.palettier.ai

import be.asmolabs.palettier.domain.image.PixelMap
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.image.ImageDecoder
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * Ce que le service dit au modele.
 *
 * <p>Tout se verifie hors ligne : la question posee est construite par l'application, et
 * c'est elle qui decide si les teintes relevees sont des cibles ou un etat de depart.</p>
 */
class PaintingPlanServiceTest {

    /** Retient la demande au lieu de l'envoyer, et rend un plan minimal. */
    private class Recorder : ChatEngine {
        override val name = "enregistreur"
        var last: JsonRequest? = null

        override suspend fun ask(request: JsonRequest): String {
            last = request
            return Json.encodeToString(
                PlanDraft(
                    approach = "Palette courte.",
                    zones = listOf(
                        ZoneDraft(
                            name = "Visage", material = "Peau", note = "",
                            base = LayerDraft("#C98F72", "Glacis", ""),
                            shadow1 = LayerDraft("#8A5F4A", "Glacis", ""),
                            shadow2 = LayerDraft("#5A3B2E", "Glacis", ""),
                            highlight1 = LayerDraft("#E0B49A", "Glacis", ""),
                            highlight2 = LayerDraft("#F2D6C0", "Glacis", ""),
                        )
                    ),
                )
            )
        }
    }

    /** Une image unie, sans plateforme : le decodage n'est pas le sujet ici. */
    private class Plain(private val argb: Int) : ImageDecoder {
        var scaled = 0
        override suspend fun decode(bytes: ByteArray) =
            PixelMap(8, 8, IntArray(64) { argb })

        override suspend fun scaleTo(bytes: ByteArray, maxEdge: Int): ByteArray {
            scaled++
            return bytes
        }
    }

    private val palette = Palette(
        name = "Zorn",
        paints = listOf(
            Paint(brand = "Gamblin", name = "Titanium White", hexColor = "#F4F2EC",
                pigments = setOf("PW6"), dryingClass = DryingClass.SLOW),
            Paint(brand = "Gamblin", name = "Ivory Black", hexColor = "#221F1C",
                pigments = setOf("PBk9"), dryingClass = DryingClass.VERY_SLOW),
            Paint(brand = "Gamblin", name = "Yellow Ochre", hexColor = "#B07C2A",
                pigments = setOf("PY43"), dryingClass = DryingClass.MEDIUM),
            Paint(brand = "Gamblin", name = "Cadmium Red", hexColor = "#B3232A",
                pigments = setOf("PR108"), dryingClass = DryingClass.VERY_SLOW),
        ),
    )

    private fun service(engine: ChatEngine?, decoder: ImageDecoder = Plain(0xFF808080.toInt())) =
        PaintingPlanService(ChatEngines { engine }, PlanEnricher(ColorMixService()), decoder)

    @Test
    fun `sans moteur, le service se declare indisponible`() = runTest {
        val service = service(null)
        assertFalse(service.isAvailable())
        val refus = assertFailsWith<PlanUnavailable> { service.plan("buste", palette, 3) }
        assertContains(refus.message!!, "parametres")
    }

    @Test
    fun `la palette est enumeree dans la question, tube par tube`() = runTest {
        val recorder = Recorder()
        service(recorder).plan("buste de grognard", palette, 3)

        val question = recorder.last!!.user
        assertContains(question, "Sujet a peindre : buste de grognard")
        assertContains(question, "Palette disponible, Zorn :")
        assertContains(question, "- Gamblin - Cadmium Red : #B3232A, pigments PR108, sechage")
        // Sans photo, rien de mesure ne doit apparaitre.
        assertFalse(question.contains("Teintes relevees"))
        assertFalse(question.contains("Images jointes"))
    }

    @Test
    fun `les teintes d'une reference sont annoncees comme des cibles`() = runTest {
        val recorder = Recorder()
        service(recorder).plan(
            "buste", palette, 3,
            figurine = PhotoInput(byteArrayOf(1), "la piece"),
            references = listOf(PhotoInput(byteArrayOf(2), "le rendu vise")),
        )

        val question = recorder.last!!.user
        assertContains(question, "sur la REFERENCE")
        assertContains(question, "ce sont tes cibles")
        assertContains(question, "Image 1 : la piece")
        assertContains(question, "Image 2 : le rendu vise")
        assertContains(question, "combler l'ecart")
    }

    @Test
    fun `les teintes d'une piece seule sont annoncees comme un etat de depart`() = runTest {
        val recorder = Recorder()
        service(recorder).plan(
            "buste", palette, 3,
            figurine = PhotoInput(byteArrayOf(1), "la piece"),
        )

        val question = recorder.last!!.user
        assertContains(question, "TELLE QU'ELLE EST")
        assertContains(question, "c'est l'etat de depart, pas la cible")
        assertFalse(question.contains("combler l'ecart"))
    }

    @Test
    fun `une photo illisible ne coute pas le plan`() = runTest {
        val recorder = Recorder()
        val cassee = object : ImageDecoder {
            override suspend fun decode(bytes: ByteArray): PixelMap =
                throw IllegalArgumentException("format inconnu")
            override suspend fun scaleTo(bytes: ByteArray, maxEdge: Int) = bytes
        }

        val plan = service(recorder, cassee)
            .plan("buste", palette, 3, figurine = PhotoInput(byteArrayOf(1), "la piece"))

        assertEquals(1, plan.zones.size)
        assertFalse(recorder.last!!.user.contains("Teintes relevees"))
        // La photo part quand meme : le modele la voit, seule la mesure manque.
        assertEquals(1, recorder.last!!.photos.size)
    }

    @Test
    fun `chaque photo soumise est reduite avant l'envoi`() = runTest {
        val decoder = Plain(0xFF404040.toInt())
        service(Recorder(), decoder).plan(
            "buste", palette, 3,
            figurine = PhotoInput(byteArrayOf(1), "la piece"),
            references = listOf(PhotoInput(byteArrayOf(2), "reference")),
        )
        assertEquals(2, decoder.scaled)
    }

    @Test
    fun `le plan rendu porte les melanges calcules, pas ceux du modele`() = runTest {
        val plan = service(Recorder()).plan("buste", palette, 3)

        val base = plan.zones.first().base!!
        assertTrue(base.recipe != null, "la base doit porter un melange calcule")
        assertTrue(base.deltaE >= 0.0)
        assertEquals("Zorn", plan.paletteName)
    }

    @Test
    fun `une reponse qui n'est pas un plan donne un message exploitable`() = runTest {
        val bavard = object : ChatEngine {
            override val name = "bavard"
            // Un modele a mode de reflexion repond parfois a cote, malgre la contrainte.
            override suspend fun ask(request: JsonRequest) = "Voici mon raisonnement : d'abord..."
        }

        val refus = assertFailsWith<PlanUnavailable> {
            service(bavard).plan("buste", palette, 3)
        }
        assertContains(refus.message!!, "plan exploitable")
    }
}
