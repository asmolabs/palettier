package be.asmolabs.palettier.domain.port

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectPhoto
import kotlinx.coroutines.flow.Flow

/**
 * Ce que le domaine attend d'un stockage, sans rien savoir de sa nature.
 *
 * <p>Les lectures continues sont des Flow : c'est la base qui previent quand elle a
 * change, et non l'ecran qui redemande. C'est ce qui supprime les rafraichissements
 * manuels que l'application JavaFX devait cabler un par un.</p>
 */
interface PaintCatalogRepository {

    fun observeAll(): Flow<List<Paint>>

    suspend fun all(): List<Paint>

    /** Les tubes reellement sur l'etagere du peintre. */
    suspend fun inStock(): List<Paint>

    suspend fun findById(id: Long): Paint?

    /**
     * Le tube designe par sa marque et son nom.
     *
     * <p>C'est la cle que porte une sauvegarde : un identifiant de base ne veut rien dire
     * dans une autre installation. C'est aussi ce qui rend l'import idempotent.</p>
     */
    suspend fun findByNaturalKey(brand: String, name: String): Paint?

    /** Enregistre un tube, en creation comme en modification. */
    suspend fun save(paint: Paint): Paint

    suspend fun setOwned(paint: Paint, owned: Boolean)

    /** Consigne la teinte diluee relevee sur un ecouvillon. */
    suspend fun recordTint(paint: Paint, tint: Rgb)

    suspend fun countOwned(): Long
}

interface ProjectRepository {

    /** Les projets sans leurs photos : ce sont des images entieres, la liste n'en a que faire. */
    fun observeAll(): Flow<List<Project>>

    suspend fun all(): List<Project>

    /** Le projet avec ses photos, pour l'ecran qui les affiche. */
    suspend fun findWithPhotos(id: Long): Project?

    /**
     * Enregistre le projet : ses zones, ses couches, ses tubes figes.
     *
     * <p>Les photos ne sont jamais touchees ici. C'est structurel, pas un oubli : un
     * projet lu sans ses photos et reenregistre les effacerait, et c'est exactement le
     * defaut qu'il a fallu corriger cote JPA. Les photos ont leurs propres methodes.</p>
     */
    suspend fun save(project: Project): Project

    suspend fun addPhoto(project: Project, photo: ProjectPhoto): Project

    suspend fun removePhoto(project: Project, photoId: Long): Project

    suspend fun delete(project: Project)
}

interface PaletteRepository {

    fun observeAll(): Flow<List<Palette>>

    suspend fun all(): List<Palette>

    suspend fun findById(id: Long): Palette?

    suspend fun save(palette: Palette): Palette

    suspend fun delete(palette: Palette)
}
