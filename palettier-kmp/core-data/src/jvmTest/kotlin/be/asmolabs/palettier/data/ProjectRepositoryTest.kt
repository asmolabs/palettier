package be.asmolabs.palettier.data

import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.paint.Ventilation
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectLayer
import be.asmolabs.palettier.domain.project.ProjectPhoto
import be.asmolabs.palettier.domain.project.ProjectZone
import be.asmolabs.palettier.domain.workbench.WorkbenchService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.days
import kotlin.time.Instant

class ProjectRepositoryTest {

    private val database = PalettierDatabase(inMemoryDriver())
    private val catalog = SqlDelightPaintCatalogRepository(database, Dispatchers.Default)
    private val palettes = SqlDelightPaletteRepository(database, catalog, Dispatchers.Default)
    private val projects = SqlDelightProjectRepository(database, catalog, palettes, Dispatchers.Default)

    private val posed = Instant.parse("2026-03-01T18:30:00Z")

    /** Le meme tube pour tous les essais : (marque, nom) est unique, et c'est voulu. */
    private suspend fun umber(): Paint =
        catalog.findByNaturalKey("Winsor & Newton", "Burnt Umber") ?: catalog.save(
            Paint(
                brand = "Winsor & Newton", name = "Burnt Umber", hexColor = "#4A3427",
                opacity = Opacity.SEMI_OPAQUE, dryingClass = DryingClass.FAST,
                tintingStrength = 0.8, pigments = setOf("PBr7"),
            )
        )

    private fun layer(role: String, hex: String) =
        ProjectLayer(role = role, targetHex = hex, technique = "Glacis")

    private suspend fun piece(name: String): Project {
        val paint = umber()
        // Une palette peut servir a plusieurs pieces : on la reutilise plutot que d'en
        // recreer une du meme nom, que la contrainte d'unicite refuserait.
        val palette = palettes.all().firstOrNull { it.name == "Zorn" }
            ?: palettes.save(Palette(name = "Zorn", paints = listOf(paint)))
        return projects.save(
            Project(
                name = name, subject = "Buste", approach = "Par glacis.",
                palette = palette, paints = listOf(paint),
                zones = listOf(
                    ProjectZone(
                        name = "Visage", material = "Peau",
                        layers = listOf(layer("Ombre 1", "#8A5F4A"), layer("Base", "#C98F72")),
                    ),
                ),
            )
        )
    }

    @Test
    fun `un projet se relit avec ses zones, ses couches et ses tubes figes`() = runTest {
        val saved = piece("Grognard")

        val read = projects.all().single()
        assertEquals("Grognard", read.name)
        assertEquals("Par glacis.", read.approach)
        assertEquals(listOf("Visage"), read.zones.map { it.name })
        assertEquals(listOf("Ombre 1", "Base"), read.zones.single().layers.map { it.role })
        assertEquals(listOf("Burnt Umber"), read.paints.map { it.name })
        assertEquals("Zorn", read.palette?.name)
        assertEquals(saved.id, read.id)
    }

    @Test
    fun `la pose d'une couche survit a la relecture, conditions comprises`() = runTest {
        val project = piece("Cape posee")
        val marked = project.copy(
            zones = project.zones.map { zone ->
                zone.copy(layers = zone.layers.mapIndexed { i, l ->
                    if (i != 0) l else l.markApplied(
                        posed, Workshop(24.0, 40.0, Ventilation.GOOD), DryingClass.VERY_SLOW
                    )
                })
            }
        )
        projects.save(marked)

        val applied = projects.all().single().zones.single().layers.first().applied
        assertNotNull(applied)
        assertEquals(posed, applied.at)
        assertEquals(24.0, applied.workshop.temperatureCelsius)
        assertEquals(40.0, applied.workshop.relativeHumidity)
        assertEquals(Ventilation.GOOD, applied.workshop.ventilation)
        assertEquals(DryingClass.VERY_SLOW, applied.dryingClass)
    }

    @Test
    fun `une couche jamais posee n'en invente pas une`() = runTest {
        piece("Rien de pose")
        assertTrue(projects.all().single().zones.single().layers.all { !it.isApplied })
    }

    @Test
    fun `enregistrer un projet lu sans ses photos ne les efface pas`() = runTest {
        // Exactement le defaut qu'il a fallu corriger cote JPA : la liste des projets ne
        // charge pas les photos, et un reenregistrement les emportait.
        var project = piece("Avec photos")
        project = projects.addPhoto(project, ProjectPhoto(data = byteArrayOf(1, 2, 3), caption = "piece.png"))
        assertEquals(1, project.photos.size)

        val fromList = projects.all().single()
        assertTrue(fromList.photos.isEmpty(), "la liste ne charge pas les photos")

        projects.save(fromList.copy(notes = "une note de plus"))

        val reread = projects.findWithPhotos(fromList.id!!)!!
        assertEquals(1, reread.photos.size, "les photos ont survecu a l'enregistrement")
        assertEquals("piece.png", reread.photos.single().caption)
        assertEquals("une note de plus", reread.notes)
    }

    @Test
    fun `une photo retiree l'est vraiment, les autres restent`() = runTest {
        var project = piece("Deux photos")
        project = projects.addPhoto(project, ProjectPhoto(data = byteArrayOf(1), caption = "piece.png"))
        project = projects.addPhoto(project, ProjectPhoto(data = byteArrayOf(2), caption = "reference.png"))
        assertEquals(listOf("piece.png", "reference.png"), project.photos.map { it.caption })

        val remaining = projects.removePhoto(project, project.photos.first().id!!)

        assertEquals(listOf("reference.png"), remaining.photos.map { it.caption })
    }

    @Test
    fun `deux projets ne peuvent pas porter le meme nom`() = runTest {
        piece("Grognard")
        piece("Grognard")

        assertEquals(setOf("Grognard", "Grognard 2"), projects.all().map { it.name }.toSet())
    }

    @Test
    fun `l'etabli se calcule directement sur ce que rend le depot`() = runTest {
        // Le but de toute la chaine : la donnee relue nourrit le domaine sans adaptation.
        val project = piece("Etabli")
        projects.save(
            project.copy(
                zones = project.zones.map { zone ->
                    zone.copy(layers = zone.layers.mapIndexed { i, l ->
                        if (i != 0) l else l.markApplied(posed, Workshop.standard(), DryingClass.FAST)
                    })
                }
            )
        )

        val bench = WorkbenchService().bench(projects.all(), posed + 30.days)
        val zone = bench.pieces.single().zones.single()

        assertTrue(zone.readyNow, "la couche est seche depuis longtemps")
        assertEquals("Base", zone.last?.nextRole)
    }

    @Test
    fun `supprimer un projet emporte ses zones et ses photos`() = runTest {
        var project = piece("A supprimer")
        project = projects.addPhoto(project, ProjectPhoto(data = byteArrayOf(7), caption = "x.png"))

        projects.delete(project)

        assertTrue(projects.all().isEmpty())
        assertNull(projects.findWithPhotos(project.id!!))
    }
}
