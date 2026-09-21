package be.asmolabs.palettier.data.catalog

import be.asmolabs.palettier.domain.catalog.CatalogLoader
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.paint.Technique
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteRepository
import be.asmolabs.palettier.domain.port.RecipeRepository
import be.asmolabs.palettier.domain.recipe.Recipe
import be.asmolabs.palettier.domain.recipe.RecipeStep

/**
 * Remplit la base au premier demarrage : le catalogue vient des fichiers de gamme, les
 * recettes et palettes d'exemple sont definies ici.
 *
 * <p>C'est ce qui manquait a l'application Kotlin. Sans lui, une installation neuve
 * restait vide indefiniment -- les six cent quatre-vingts huiles n'existaient que dans
 * les ressources du module Java, et le seul chemin pour les obtenir etait d'importer une
 * sauvegarde. Un peintre qui decouvre l'application n'en a aucune.</p>
 *
 * <p>Le catalogue n'est charge en entier que si la base est vide. Les corrections
 * apportees par le peintre a ses propres fiches ne sont donc jamais ecrasees par une mise
 * a jour des fichiers de gamme ; pour repartir des fichiers, supprimez
 * {@code ~/.palettier/}.</p>
 */
class CatalogSeeder(
    private val paints: PaintCatalogRepository,
    private val palettes: PaletteRepository,
    private val recipes: RecipeRepository,
    private val loader: CatalogLoader = CatalogLoader(),
) {

    /** Ce que le demarrage a fait, pour que l'ecran ou le journal puisse le dire. */
    data class Report(
        val paintsCreated: Int = 0,
        val paintsRefreshed: Int = 0,
        val paintsRemoved: Int = 0,
        val paintsKeptInUse: Int = 0,
        val tintsBackfilled: Int = 0,
        val palettesAdded: Int = 0,
        val recipesAdded: Int = 0,
        val unknownPigments: List<String> = emptyList(),
    ) {
        val touchedAnything: Boolean
            get() = paintsCreated > 0 || paintsRefreshed > 0 || paintsRemoved > 0 ||
                tintsBackfilled > 0 || palettesAdded > 0 || recipesAdded > 0
    }

    /**
     * A appeler une fois au demarrage, avant d'afficher quoi que ce soit.
     *
     * <p>Idempotent par construction : le deuxieme appel ne fait rien, et c'est ce qui
     * permet de l'appeler sans condition. Du temps de Spring, c'etait un ApplicationRunner
     * que le conteneur declenchait ; ici, personne ne declenche rien tout seul.</p>
     */
    suspend fun seed(): Report {
        val reference = loader.loadDetailed()

        val created = if (paints.count() == 0L) {
            paints.saveAll(reference.paints)
            reference.paints.size
        } else {
            0
        }

        val sync = if (created == 0) syncCatalogue(reference.paints) else Sync()
        val tints = backfillTints(reference.paints)

        return Report(
            paintsCreated = created + sync.added,
            paintsRefreshed = sync.refreshed,
            paintsRemoved = sync.removed,
            paintsKeptInUse = sync.keptInUse,
            tintsBackfilled = tints,
            palettesAdded = seedMissingPalettes(),
            recipesAdded = seedMissingRecipes(),
            unknownPigments = reference.unknownPigments,
        )
    }

    private class Sync(
        var refreshed: Int = 0,
        var added: Int = 0,
        var removed: Int = 0,
        var keptInUse: Int = 0,
    )

    /**
     * Aligne le catalogue de la base sur les fichiers de gamme.
     *
     * <p>Trois mouvements. Les tubes absents sont ajoutes : une gamme completee doit
     * arriver chez un peintre qui utilise deja l'application. Les tubes presents voient
     * leur <em>identite</em> rafraichie -- reference, pigments, opacite, sechage -- parce
     * qu'elle vient du fabricant et qu'une fiche erronee doit pouvoir etre corrigee par
     * une mise a jour. Enfin, les tubes d'une gamme livree qui ne figurent plus dans son
     * fichier sont retires : ce sont des fiches d'une version anterieure, parfois
     * fausses.</p>
     *
     * <p>Ce qui appartient au peintre n'est jamais touche : la teinte relevee sur ses
     * ecouvillons, la teinte diluee, la possession, ses notes, et les tubes qu'il a saisis
     * lui-meme.</p>
     */
    private suspend fun syncCatalogue(reference: List<Paint>): Sync {
        val sync = Sync()
        val fromFiles = reference.associateBy { key(it) }
        val coveredBrands = reference.mapTo(HashSet()) { it.brand }

        val stored = paints.all()
        val obsolete = mutableListOf<Paint>()

        for (paint in stored) {
            val file = fromFiles[key(paint)]
            if (file != null) {
                val refreshed = refreshIdentity(paint, file)
                if (refreshed != paint) {
                    paints.refreshIdentity(refreshed)
                    sync.refreshed++
                }
            } else if (!paint.userAdded && paint.brand in coveredBrands) {
                obsolete += paint
            }
        }

        val known = stored.mapTo(HashSet()) { key(it) }
        for ((fileKey, paint) in fromFiles) {
            if (fileKey !in known) {
                paints.save(paint)
                sync.added++
            }
        }

        removeObsolete(obsolete, sync)
        return sync
    }

    /**
     * Reprend du fichier ce qui releve du fabricant, et rien d'autre.
     *
     * @return la fiche a ecrire, identique a [stored] s'il n'y a rien a reprendre
     */
    private fun refreshIdentity(stored: Paint, reference: Paint): Paint {
        val updated = stored.copy(
            code = reference.code,
            legacyCode = reference.legacyCode,
            pigments = reference.pigments,
            opacity = reference.opacity,
            dryingClass = reference.dryingClass,
            tintingStrength = reference.tintingStrength,
            pigmentsVerified = reference.pigmentsVerified,
        )
        // La teinte relevee par le peintre l'emporte ; celle deduite se laisse corriger.
        return if (updated.colorDerived && !reference.colorDerived) {
            updated.copy(hexColor = reference.hexColor, colorDerived = false)
        } else {
            updated
        }
    }

    /**
     * Retire les fiches perimees, en epargnant celles qui servent encore.
     *
     * <p>Un tube employe par une palette ou fige dans un projet ne peut pas disparaitre
     * sans emporter ce qui s'y refere. On demande donc d'abord lesquels sont utilises,
     * plutot que de tenter la suppression pour voir : la contrainte cederait et
     * emporterait la palette avec elle.</p>
     */
    private suspend fun removeObsolete(obsolete: List<Paint>, sync: Sync) {
        if (obsolete.isEmpty()) return
        val inUse = paints.idsInUse()

        for (paint in obsolete) {
            if (paint.id in inUse) {
                sync.keptInUse++
            } else {
                paints.delete(paint)
                sync.removed++
            }
        }
    }

    /**
     * Renseigne la teinte diluee des huiles qui n'en ont pas encore.
     *
     * <p>Ces teintes sont arrivees apres coup, avec le passage au modele de melange a deux
     * constantes. Les bases deja en service n'en ont donc aucune, et sans ce rattrapage
     * leurs proprietaires resteraient sur l'ancien modele indefiniment. Seule la teinte
     * est ecrite : tout ce que le peintre a pu corriger par ailleurs est laisse en
     * place.</p>
     */
    private suspend fun backfillTints(reference: List<Paint>): Int {
        val fromFiles = reference.mapNotNull { paint ->
            paint.tintHex?.let { key(paint) to it }
        }.toMap()

        var updated = 0
        for (paint in paints.all()) {
            if (paint.tintHex != null) continue
            val tint = fromFiles[key(paint)] ?: continue
            paints.recordTint(paint, Rgb.ofHex(tint))
            updated++
        }
        return updated
    }

    private fun key(paint: Paint): String = "${paint.brand}|${paint.name}"

    /**
     * Ajoute les palettes de reference absentes, une par une.
     *
     * <p>Volontairement pas un "tout ou rien" sur une base vide : une palette de reference
     * ajoutee dans une version ulterieure doit pouvoir arriver chez un peintre qui utilise
     * deja l'application, sans toucher a celles qu'il a composees lui-meme. Une palette
     * supprimee expres reapparaitra au demarrage suivant : renommez-la plutot que de la
     * supprimer si elle vous gene.</p>
     */
    private suspend fun seedMissingPalettes(): Int {
        val existing = palettes.all().mapTo(HashSet()) { it.name.lowercase() }
        var added = 0
        for (candidate in referencePalettes()) {
            if (candidate.name.lowercase() in existing) continue
            palettes.save(candidate)
            added++
        }
        return added
    }

    private suspend fun seedMissingRecipes(): Int {
        var added = 0
        for (candidate in defaultRecipes()) {
            if (recipes.exists(candidate.name)) continue
            recipes.save(candidate)
            added++
        }
        return added
    }

    // --- Contenu d'exemple -------------------------------------------------

    private suspend fun referencePalettes(): List<Palette> = listOf(zorn(), flesh(), armour())

    /**
     * La palette de Zorn : quatre tubes, tous chez Winsor & Newton.
     *
     * <p>Son interet n'est pas l'economie mais la coherence. Le noir d'ivoire n'y joue pas
     * le role d'un noir : sorti du tube il est franchement chaud, mais coupe de blanc il
     * donne des gris bleutes, et mele a l'ocre des verts sourds. C'est lui qui tient le
     * role du bleu, et par lui que passent toutes les ombres. La palette couvre ainsi les
     * quatre quadrants chromatiques sans qu'aucune couleur criarde soit atteignable, ce
     * qui la rend redoutable sur les carnations.</p>
     *
     * <p>Zorn employait du vermillon, pigment au mercure aujourd'hui abandonne : le
     * cadmium ecarlate en est le substitut usuel. Attention, c'est lui qui impose le
     * rythme de la palette, les cadmiums etant les pigments les plus lents a secher.</p>
     */
    private suspend fun zorn() = Palette(
        name = "Palette Zorn (Winsor & Newton)",
        purpose = "Carnations et portraits",
        notes = """
            Quatre tubes : blanc de titane, ocre jaune, cadmium ecarlate, noir d'ivoire.

            Le noir ne sert pas a noircir, il tient le role du bleu. Sorti du tube il est chaud ; coupe de blanc il donne des gris franchement bleutes, et mele a l'ocre des verts sourds. Les ombres d'une carnation se font donc au noir, sans jamais ajouter de bleu.

            Cette divergence entre le tube et le melange est reelle, et c'est ce que le modele a deux constantes de l'application sait reproduire.

            Zorn peignait au vermillon ; le cadmium ecarlate le remplace. C'est le pigment le plus lent de la palette, il commande le planning de la seance.
        """.trimIndent(),
        paints = resolve(
            "Winsor & Newton" to "Titanium White",
            "Winsor & Newton" to "Yellow Ochre",
            "Winsor & Newton" to "Cadmium Scarlet",
            "Winsor & Newton" to "Ivory Black",
        ),
    )

    private suspend fun flesh() = Palette(
        name = "Carnations 1/10",
        purpose = "Visages et mains de buste",
        notes = "Le blanc et le rouge portent les lumieres, la terre d'ombre et le bleu de " +
            "Prusse portent les ombres. Tout le reste se trouve en melangeant ces deux " +
            "extremites : une palette courte donne des carnations plus coherentes qu'une " +
            "palette fournie.",
        paints = resolve(
            "Abteilung 502" to "Basic Flesh Tone",
            "Abteilung 502" to "Flesh Shadow",
            "Abteilung 502" to "Light Flesh Tone",
            "Winsor & Newton" to "Naples Yellow",
            "Winsor & Newton" to "Alizarin Crimson",
            "Winsor & Newton" to "Burnt Umber",
            "Winsor & Newton" to "Prussian Blue",
            "Winsor & Newton" to "Titanium White",
        ),
    )

    private suspend fun armour() = Palette(
        name = "Blindage vert olive",
        purpose = "Vehicule 1/35",
        notes = "Filtres, jus et salissures. Le blanc ne sert qu'a casser le vert, jamais a eclaircir.",
        paints = resolve(
            "Winsor & Newton" to "Olive Green",
            "Winsor & Newton" to "Yellow Ochre",
            "Winsor & Newton" to "Ivory Black",
            "Abteilung 502" to "Cassel Earth/Shadow Brown",
            "Abteilung 502" to "Starship Filth",
            "Abteilung 502" to "Red Ochre/Light Rust",
            "Gamblin" to "Titanium White",
        ),
    )

    /**
     * Les tubes cites, dans l'ordre, en passant les introuvables.
     *
     * <p>Une palette d'exemple incomplete vaut mieux qu'un demarrage bloque parce qu'un
     * nom de tube a change chez un fabricant.</p>
     */
    private suspend fun resolve(vararg keys: Pair<String, String>): List<Paint> =
        keys.mapNotNull { (brand, name) -> paints.findByNaturalKey(brand, name) }

    private fun defaultRecipes(): List<Recipe> = listOf(
        Recipe(
            name = "Carnation 1/10 a l'huile",
            subject = "Visage de buste",
            notes = """
                Base acrylique poncee et vernie brillante, seche depuis 24 h.
                Tout le modele repose sur des couches tres fines : on construit la profondeur
                par accumulation, jamais en une seule passe chargee.
            """.trimIndent(),
            steps = listOf(
                step(
                    Technique.BASE_LAYER, "Basic Flesh Tone + une pointe de Naples Yellow",
                    Medium.ODORLESS_THINNER, 0.25, LayerThickness.THIN,
                    "Aplat tres fin, on doit encore deviner la base acrylique.",
                ),
                step(
                    Technique.DOT_FADING, "Points de Deep Shadow Flesh, Naples Yellow et Alizarin Crimson",
                    Medium.NONE, 0.0, LayerThickness.THIN,
                    "Rouge sur les pommettes, le nez et les oreilles ; jaune sur le front.",
                ),
                step(
                    Technique.BLENDING, "Fondu au pinceau propre",
                    Medium.NONE, 0.0, LayerThickness.THIN,
                    "Tirer les transitions tant que la couche est ouverte, essuyer le pinceau a chaque passage.",
                ),
                step(
                    Technique.PIN_WASH, "Burnt Umber + une pointe de Prussian Blue",
                    Medium.ODORLESS_THINNER, 0.90, LayerThickness.GLAZE,
                    "Creux des paupieres, ailes du nez, commissures.",
                ),
                step(
                    Technique.HIGHLIGHT, "Snow White + Basic Flesh Tone",
                    Medium.NONE, 0.0, LayerThickness.THIN,
                    "Arete du nez, arcade, levre inferieure. Trois points suffisent.",
                ),
            ),
        ),
        Recipe(
            name = "Usure d'un blindage vert olive",
            subject = "Vehicule 1/35",
            notes = "Enchainement classique : filtre, jus, coulures, puis poussieres.",
            steps = listOf(
                step(
                    Technique.FILTER, "Olive Green tres dilue",
                    Medium.ODORLESS_THINNER, 0.94, LayerThickness.GLAZE,
                    "Unifie les modules de camouflage sans les effacer.",
                ),
                step(
                    Technique.PIN_WASH, "Cassel Earth/Shadow Brown",
                    Medium.ODORLESS_THINNER, 0.90, LayerThickness.GLAZE,
                    "Uniquement dans les lignes de structure et autour des rivets.",
                ),
                step(
                    Technique.STREAKING_GRIME, "Starship Filth",
                    Medium.ODORLESS_THINNER, 0.75, LayerThickness.THIN,
                    "Traits verticaux depuis les asperites, tires vers le bas apres deux minutes.",
                ),
                step(
                    Technique.GLAZE, "Rust dans les zones d'ecaillage",
                    Medium.ODORLESS_THINNER, 0.85, LayerThickness.GLAZE,
                    "Toujours au-dessus de l'ecaillage, jamais en dessous.",
                ),
            ),
        ),
    )

    private fun step(
        technique: Technique,
        mix: String,
        medium: Medium,
        ratio: Double,
        thickness: LayerThickness,
        notes: String,
    ) = RecipeStep(
        technique = technique,
        paintMix = mix,
        medium = medium,
        mediumRatio = ratio,
        thickness = thickness,
        notes = notes,
    )
}
