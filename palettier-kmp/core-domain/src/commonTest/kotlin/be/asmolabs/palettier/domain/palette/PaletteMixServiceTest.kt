package be.asmolabs.palettier.domain.palette

import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Ventilation
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlin.time.Duration
import kotlin.time.Duration.Companion.days
import kotlin.time.Duration.Companion.hours
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Instant

class PaletteMixServiceTest {

    private val now = Instant.parse("2026-03-01T10:00:00Z")
    private val service = PaletteMixService()

    private fun gris(drying: DryingClass = DryingClass.MEDIUM, workshop: Workshop = Workshop.standard()) =
        PaletteMix(
            name = "Gris rompu des ombres", hexColor = "#6B6259",
            recipe = "2 parts de terre d'ombre + 1 part de blanc",
            dryingClass = drying, mixedAt = now, workshop = workshop,
        )

    @Test
    fun `un melange frais est ouvert, et le restera quelques heures`() {
        val state = service.state(gris(), now + 30.minutes)

        assertTrue(state.isOpen)
        assertTrue(!state.isSpent)
        assertTrue(state.workable > Duration.ZERO)
    }

    @Test
    fun `passe le temps ouvert il ne se travaille plus, et plus tard il a pris`() {
        val spent = service.state(gris(DryingClass.FAST), now + 20.days)

        assertTrue(!spent.isOpen)
        assertTrue(spent.isSpent)
    }

    @Test
    fun `le temps ouvert depend de l'atelier du moment ou l'on a melange`() {
        val cold = service.state(gris(workshop = Workshop(10.0, 80.0, Ventilation.CONFINED)), now + 1.hours)
        val warm = service.state(gris(workshop = Workshop(28.0, 30.0, Ventilation.GOOD)), now + 1.hours)

        assertTrue(cold.workable > warm.workable, "un atelier froid et humide garde le melange ouvert plus longtemps")
    }

    @Test
    fun `une pate sur la palette tient plus longtemps que la meme couche sur la piece`() {
        assertTrue(service.state(gris(), now).workable > 1.hours, "le tas n'est pas etale")
    }

    @Test
    fun `le menage ne retire que ce qui a pris`() {
        val mixes = listOf(gris())

        assertEquals(emptyList(), service.spent(mixes, now))
        assertEquals(1, service.spent(mixes, now + 21.days).size)
    }
}
