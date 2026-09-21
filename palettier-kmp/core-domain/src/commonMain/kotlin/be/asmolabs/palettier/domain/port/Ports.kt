package be.asmolabs.palettier.domain.port

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.palette.PaletteMix
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectPhoto
import be.asmolabs.palettier.domain.recipe.Recipe
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

    /**
     * Enregistre un lot d'un seul coup.
     *
     * <p>C'est l'amorcage du catalogue qui l'impose : six cent quatre-vingts appels a
     * [save] font autant de transactions, et le premier demarrage s'en ressent.</p>
     */
    suspend fun saveAll(paints: List<Paint>)

    suspend fun setOwned(paint: Paint, owned: Boolean)

    /** Consigne la teinte diluee relevee sur un ecouvillon. */
    suspend fun recordTint(paint: Paint, tint: Rgb)

    suspend fun countOwned(): Long

    suspend fun count(): Long

    /**
     * Reprend du fichier de gamme ce qui releve du fabricant, et rien d'autre.
     *
     * <p>Reference, pigments, opacite, sechage : ces champs viennent du fabricant, et une
     * fiche erronee doit pouvoir etre corrigee par une mise a jour de l'application. Ce
     * que le peintre a releve -- sa teinte d'ecouvillon, sa possession, ses notes -- n'est
     * jamais touche ici, sinon une correction du catalogue effacerait son travail.</p>
     */
    suspend fun refreshIdentity(paint: Paint)

    /** Les identifiants de tubes qu'une palette ou un projet cite encore. */
    suspend fun idsInUse(): Set<Long>

    suspend fun delete(paint: Paint)
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

interface PaletteMixRepository {

    fun observeAll(): Flow<List<PaletteMix>>

    suspend fun all(): List<PaletteMix>

    suspend fun save(mix: PaletteMix): PaletteMix

    suspend fun delete(id: Long)
}

interface RecipeRepository {

    fun observeAll(): Flow<List<Recipe>>

    suspend fun all(): List<Recipe>

    suspend fun exists(name: String): Boolean

    suspend fun save(recipe: Recipe): Recipe

    suspend fun delete(recipe: Recipe)
}

interface PaletteRepository {

    fun observeAll(): Flow<List<Palette>>

    suspend fun all(): List<Palette>

    suspend fun findById(id: Long): Palette?

    suspend fun save(palette: Palette): Palette

    suspend fun delete(palette: Palette)
}

/**
 * Les reglages du peintre, en clef et valeur.
 *
 * <p>Du temps de Spring, le choix du moteur d'assistance vivait dans un fichier de
 * configuration lu au demarrage. Sur un telephone, il n'y a pas de fichier a editer : le
 * reglage se choisit dans l'application, et doit donc se ranger quelque part.</p>
 */
interface SettingsRepository {

    fun observe(key: String): Flow<String?>

    suspend fun get(key: String): String?

    suspend fun put(key: String, value: String)

    suspend fun remove(key: String)
}
