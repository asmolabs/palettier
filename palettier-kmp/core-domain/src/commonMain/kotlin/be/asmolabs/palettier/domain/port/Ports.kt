package be.asmolabs.palettier.domain.port

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.project.Project
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

    suspend fun save(project: Project): Project

    suspend fun delete(project: Project)
}
