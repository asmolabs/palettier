package be.asmolabs.palettier.ai

import be.asmolabs.palettier.domain.image.ImagePalette
import be.asmolabs.palettier.domain.project.DominantColour
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.plan.PaintingPlan
import be.asmolabs.palettier.image.ImageDecoder
import kotlinx.serialization.json.Json

/**
 * Etablit un plan de peinture par zones pour un sujet donne, avec la palette du peintre.
 *
 * <p>Le travail se fait en deux temps, et cette separation est tout l'interet du
 * service. Le modele de langage enumere les zones du sujet et choisit, pour chacune, la
 * teinte de base, l'ombre et la lumiere : ce sont des jugements de peintre, que rien
 * dans l'application ne sait produire. Puis l'application calcule, pour chaque couleur
 * visee, le melange qui l'approche le mieux avec les tubes reellement disponibles.</p>
 *
 * <p>Le modele n'ecrit donc aucun dosage et l'ecart annonce est mesure, pas affirme. Si
 * la palette ne permet pas d'atteindre une couleur, cela se voit : c'est une information
 * utile, pas un echec a masquer.</p>
 */
class PaintingPlanService(
    private val engines: ChatEngines,
    private val enricher: PlanEnricher,
    private val decoder: ImageDecoder,
) {

    /**
     * Vrai si un moteur de conversation est configure. Sans cela le reste de
     * l'application fonctionne a l'identique : l'assistant est un supplement, pas une
     * dependance.
     */
    private val json = Json { ignoreUnknownKeys = true }

    suspend fun isAvailable(): Boolean = engines.current() != null

    /**
     * @param subject    description libre du sujet, par exemple "buste de grognard
     *                   napoleonien, manteau bleu et bonnet a poil"
     * @param palette    palette avec laquelle le plan doit etre realisable
     * @param maxPaints  nombre maximal de tubes par melange
     * @param figurine   photo de la piece a peindre, ou null
     * @param references photos de ce que l'on cherche a obtenir, eventuellement vide
     * @param model      modele a employer pour cette demande, ou null pour celui du moteur
     */
    suspend fun plan(
        subject: String,
        palette: Palette,
        maxPaints: Int,
        figurine: PhotoInput? = null,
        references: List<PhotoInput> = emptyList(),
        model: String? = null,
    ): PaintingPlan {
        val engine = engines.current() ?: throw PlanUnavailable(
            "Aucun moteur de conversation configure. Choisissez-en un dans les parametres."
        )

        // Reduction imposee ici, et non laissee a l'appelant : c'est une contrainte du
        // transport, pas un choix d'interface.
        val photos = buildList {
            figurine?.let { add(reduced(it)) }
            references.forEach { add(reduced(it)) }
        }

        val question = userPrompt(
            subject, palette, figurine != null, photos,
            measuredColours(figurine, references),
        )

        val answer = engine.ask(
            JsonRequest(SYSTEM_PROMPT, question, PlanSchema.plan, photos, model)
        )
        return enricher.enrich(subject, palette, answer.toPlanDraft(json), maxPaints)
    }

    private suspend fun reduced(photo: PhotoInput) =
        PhotoInput(decoder.scaleTo(photo.data, SUBMITTED_EDGE), photo.caption)

    /**
     * Les teintes reellement presentes sur les photos fournies.
     *
     * <p>C'est la difference entre un plan fonde sur la piece et un plan fonde sur l'idee
     * qu'on se fait du sujet. La reference l'emporte sur l'etat actuel quand les deux sont
     * la : on peint vers ce qu'on veut obtenir.</p>
     */
    private data class Measured(
        val colours: List<DominantColour>,
        val fromReference: Boolean,
    ) {
        val isEmpty: Boolean get() = colours.isEmpty()

        companion object {
            val NONE = Measured(emptyList(), false)
        }
    }

    private suspend fun measuredColours(figurine: PhotoInput?, references: List<PhotoInput>): Measured {
        val hasReference = references.isNotEmpty()
        val source = if (hasReference) references.first().data else figurine?.data ?: return Measured.NONE
        return try {
            Measured(ImagePalette.dominant(decoder.decode(source), MEASURED_COLOURS), hasReference)
        } catch (e: IllegalArgumentException) {
            // Une photo illisible ne doit pas couter le plan : on s'en passe.
            Measured.NONE
        }
    }

    /**
     * La palette est enumeree dans la question elle-meme plutot que laissee a un appel
     * d'outil : c'est l'information dont le modele a besoin a coup sur, autant la lui
     * donner d'emblee que dependre de son initiative.
     */
    private fun userPrompt(
        subject: String,
        palette: Palette,
        hasFigurine: Boolean,
        photos: List<PhotoInput>,
        measured: Measured,
    ): String = buildString {
        if (!measured.isEmpty) {
            // La distinction est decisive : les teintes d'une reference sont des cibles,
            // celles d'une piece nue ne sont que du metal ou de l'appret.
            append(
                if (measured.fromReference) {
                    "Teintes relevees par l'application sur la REFERENCE, avec la part de " +
                        "l'image qu'elles occupent. Ce sont des mesures, et ce sont tes cibles :\n"
                } else {
                    "Teintes relevees par l'application sur la PIECE TELLE QU'ELLE EST, avec la " +
                        "part de l'image qu'elles occupent :\n"
                }
            )
            measured.colours.forEach { colour ->
                append("  ").append(colour.color.toHex())
                    .append("  ").append((colour.share * 100).toInt()).append(" %\n")
            }
            append(
                if (measured.fromReference) {
                    "Appuie tes couleurs sur ces mesures plutot que sur l'idee generale que tu " +
                        "te fais du sujet.\n"
                } else {
                    "Attention : c'est l'etat de depart, pas la cible. Si la piece est nue -- " +
                        "metal, resine, appret gris -- ces teintes ne disent rien de ce qu'il faut " +
                        "peindre, et tu dois les ignorer completement pour proposer les couleurs " +
                        "du sujet represente. Elles ne servent que si la piece est deja peinte.\n"
                }
            )
            append("Une part tres importante correspond souvent au fond, ignore-la.\n\n")
        }

        // Les images arrivent dans l'ordre ou elles sont ajoutees, sans etiquette : sans
        // cette enumeration, le modele ne sait pas laquelle est la piece et laquelle est
        // la reference.
        if (photos.isNotEmpty()) {
            append("Images jointes, dans l'ordre :\n")
            photos.forEachIndexed { i, photo ->
                append("  Image ").append(i + 1).append(" : ").append(photo.caption).append('\n')
            }
            append('\n')
            if (hasFigurine && photos.size > 1) {
                append(
                    "Compare l'etat actuel de la piece aux references, et batis le " +
                        "plan pour combler l'ecart.\n\n"
                )
            } else if (hasFigurine) {
                append("Decoupe la piece en zones d'apres ce que tu vois.\n\n")
            }
        }

        append("Sujet a peindre : ")
            .append(if (subject.isBlank()) "voir les images jointes" else subject)
            .append("\n\n")
        append("Palette disponible, ").append(palette.name).append(" :\n")
        palette.paints.forEach { paint ->
            append("- ").append(paint.displayName)
                .append(" : ").append(paint.hexColor)
                .append(", pigments ").append(paint.pigments.joinToString(" "))
                .append(", sechage ").append(paint.dryingClass.label.lowercase())
                .append('\n')
        }
        append("\nDecompose le sujet en zones et donne pour chacune la base, l'ombre et la lumiere.")
    }

    companion object {

        /** Nombre de teintes relevees sur la photo et soumises au modele. */
        const val MEASURED_COLOURS = 10

        /** Cote maximal des images soumises : au-dela, on paie du transport pour rien. */
        const val SUBMITTED_EDGE = 768

        val SYSTEM_PROMPT = """
            Tu es un peintre sur figurine confirme, specialiste de la peinture a l'huile.

            On te donne un sujet et la liste exacte des tubes dont dispose le peintre. Tu
            decomposes le sujet en zones (peau, cheveux, tissus, cuir, metal, bois, base...)
            et, pour chaque zone, tu remplis cinq couches : base, shadow1, shadow2,
            highlight1, highlight2.

            shadow1 est une ombre legere dans les demi-tons, shadow2 l'ombre profonde des
            creux fermes ; highlight1 est le premier eclairci sur les volumes exposes,
            highlight2 le point lumineux, pose sur une arete seulement.

            Ces cinq couches ne forment qu'une echelle de valeurs, du sombre au clair. Une
            zone en demande presque toujours davantage : des couleurs qui ne sont ni plus
            claires ni plus sombres, mais AUTRES. Remplis donc aussi accent1, accent2 et
            accent3 avec ces variations locales, a peu pres a la valeur de la base.

            Sur une carnation, ce sont elles qui font la difference entre une peau correcte
            et une peau vivante : les pommettes, le nez et les oreilles tirent au rouge, le
            front au jaune, la machoire et les tempes au froid ou au vert, le tour des yeux
            au violet. Sur un tissu, ce sera un reflet de couleur voisine ; sur un metal,
            une trace de rouille ou de chaleur.

            Chaque accent porte un nom qui dit OU le poser -- "rougeur des pommettes",
            "front plus jaune" -- et non un rang. Laisse un accent vide quand la zone n'en
            demande pas : un aplat de bois sec n'a pas de variation locale. Une seule ombre
            et une seule lumiere suffisent a poser un volume, jamais a le rendre : c'est
            l'etagement des valeurs qui separe une piece plate d'une piece modelee. La
            derniere ombre se loge dans les creux les plus fermes, le dernier point lumineux
            sur une arete seulement.

            Regles imperatives :
            - Chaque couleur est donnee en hexadecimal sRGB, au format #RRGGBB. Jamais un nom.
            - NE RECOPIE JAMAIS la couleur d'un tube telle qu'elle t'est donnee. Les couleurs
              que tu vises sont des melanges : elles tombent ENTRE les tubes, jamais dessus.
              Une teinte visee identique a celle d'un tube de la liste est une reponse ratee,
              meme si elle a l'air propre.

            - Une ombre n'est pas la base assombrie au noir. Elle est plus froide, plus
              rompue, et garde la couleur locale. Une lumiere n'est pas du blanc : elle est
              teintee, et souvent plus chaude que la base.

            - L'ecart de valeur entre l'ombre et la lumiere reste mesure. Une figurine peinte
              du noir au blanc pur parait crayeuse ; l'essentiel du modele vit dans les tons
              moyens.

            - Ne propose que des couleurs atteignables avec les tubes fournis. Une palette
              courte impose des teintes rompues : c'est normal, ne l'ignore pas.
            - La technique de chaque couche est un de ces noms exactement :
              Aplat de base, Jus a l'huile, Jus capillaire (pin wash), Filtre, Glacis,
              Fondu / degrade, Dot fading, Coulures et salissures, Oil Paint Rendering (OPR),
              Eclaircis et points lumineux.
            - N'indique aucun dosage, aucune proportion, aucune duree de sechage : ces
              nombres sont calcules par l'application, pas par toi.
            - Six zones au maximum, et regroupe ce qui se peint pareil. Mieux vaut cinq zones
              completes que douze bacles : chaque zone porte cinq couches, la reponse est vite
              longue.

            - Les notes sont tres breves : une dizaine de mots, ou poser la couche ou quel
              ecueil eviter. Pas de phrase d'introduction, pas de redite d'une zone a l'autre.

            Si des photos t'accompagnent, decoupe le sujet d'apres ce que tu vois sur la
            piece, pas d'apres ce qu'un tel sujet comporte en general.

            Attention au cas frequent de la piece nue, encore en metal, en resine ou en
            simple appret gris : elle n'est pas grise, elle est SANS PEINTURE. Tu dois
            alors planifier d'apres ce que la sculpture represente -- le casque, la
            tunique, le ceinturon, les bottes, le paquetage, l'arme, la peau du visage et
            des mains -- et nommer ces zones-la. Une reponse qui se contenterait de
            "metal", "tissu" et "salissures" serait inutilisable : le peintre a besoin de
            savoir quoi peindre, piece d'equipement par piece d'equipement.

            Quand des teintes mesurees sur la photo te sont fournies, elles font autorite.
            Elles ont ete relevees par l'application, pas estimees : tes couleurs doivent
            s'appuyer dessus, et non sur l'idee generale que tu te fais du sujet. Un visage
            n'est pas "de la couleur chair", c'est les teintes relevees sur CE visage.
            Tu peux les eclaircir, les assombrir ou les rompre pour construire le modele,
            mais tu ne dois pas partir ailleurs.
        """.trimIndent()
    }
}
