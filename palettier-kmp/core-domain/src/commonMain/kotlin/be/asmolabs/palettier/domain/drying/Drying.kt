package be.asmolabs.palettier.domain.drying

import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import be.asmolabs.palettier.domain.paint.Technique
import be.asmolabs.palettier.domain.paint.Ventilation
import kotlin.math.pow
import kotlin.math.roundToLong
import kotlin.time.Duration
import kotlin.time.Duration.Companion.minutes

/** Conditions de l'atelier. */
data class Workshop(
    val temperatureCelsius: Double,
    val relativeHumidity: Double,
    val ventilation: Ventilation,
) {
    companion object {
        fun standard() = Workshop(20.0, 50.0, Ventilation.NORMAL)
    }
}

/** Tout ce qui determine le sechage d'une couche. */
data class DryingContext(
    val dryingClass: DryingClass,
    val medium: Medium,
    val mediumRatio: Double,
    val thickness: LayerThickness,
    val workshop: Workshop,
) {
    init {
        require(mediumRatio in 0.0..1.0) { "Proportion de medium hors de [0, 1] : $mediumRatio" }
    }

    companion object {
        /** Contexte pre-rempli avec les reglages usuels de la technique. */
        fun forTechnique(technique: Technique, dryingClass: DryingClass, workshop: Workshop) =
            DryingContext(
                dryingClass, technique.defaultMedium, technique.defaultRatio,
                technique.typicalThickness, workshop,
            )
    }
}

/**
 * Jalons de sechage d'une couche.
 *
 * @param openTime   duree pendant laquelle la couche reste travaillable (fondus, dot fading)
 * @param touchDry   sec au toucher, la couche ne marque plus
 * @param recoat     on peut poser la couche suivante sans relever celle-ci
 * @param throughDry sec a coeur, manipulation et masquage possibles
 * @param fullCure   polymerisation complete, avant vernis final
 */
data class DryingEstimate(
    val openTime: Duration,
    val touchDry: Duration,
    val recoat: Duration,
    val throughDry: Duration,
    val fullCure: Duration,
    val advice: List<String>,
)

/**
 * Parametres du modele de sechage.
 *
 * <p>Ils venaient d'application.yaml via @ConfigurationProperties. Il n'y a pas de
 * fichier de configuration en multiplateforme : ce sont desormais des valeurs par
 * defaut du domaine, qu'une couche superieure peut surcharger si elle le souhaite.</p>
 */
data class DryingProperties(
    val referenceTemperature: Double = 20.0,
    val temperatureHalvingStep: Double = 10.0,
    val referenceHumidity: Double = 50.0,
    val humiditySensitivity: Double = 0.6,
    val openTimeShare: Double = 0.22,
    val recoatFactor: Double = 1.8,
    val throughDryFactor: Double = 5.0,
    val fullCureFactor: Double = 22.0,
)

/**
 * Estime les jalons de sechage d'une couche d'huile.
 *
 * <p>Le modele part du temps de reference du pigment le plus lent, puis applique
 * successivement l'effet du medium, de l'epaisseur, de la temperature, de l'humidite et
 * de la ventilation. Ce sont des ordres de grandeur destines a planifier une seance, pas
 * une mesure : sur une piece reelle, on verifie toujours du doigt sur le socle.</p>
 */
class DryingTimeService(private val properties: DryingProperties = DryingProperties()) {

    fun estimate(context: DryingContext): DryingEstimate {
        val hours = context.dryingClass.referenceTouchDry.inWholeMinutes / 60.0 *
            context.medium.dryingFactor(context.mediumRatio) *
            context.thickness.factor *
            temperatureFactor(context.workshop.temperatureCelsius) *
            humidityFactor(context.workshop.relativeHumidity) *
            context.workshop.ventilation.factor

        return DryingEstimate(
            openTime = hours(hours * properties.openTimeShare),
            touchDry = hours(hours),
            recoat = hours(hours * properties.recoatFactor),
            throughDry = hours(hours * properties.throughDryFactor),
            fullCure = hours(hours * properties.fullCureFactor),
            advice = advice(context),
        )
    }

    /** La vitesse d'oxydation double environ a chaque palier de temperature gagne. */
    private fun temperatureFactor(celsius: Double): Double {
        val delta = properties.referenceTemperature - celsius
        return 2.0.pow(delta / properties.temperatureHalvingStep)
    }

    /** Une atmosphere humide ralentit la prise de l'huile, une atmosphere seche l'accelere. */
    private fun humidityFactor(relativeHumidity: Double): Double {
        val delta = relativeHumidity - properties.referenceHumidity
        return maxOf(0.5, 1.0 + delta / 100.0 * properties.humiditySensitivity)
    }

    private fun advice(context: DryingContext): List<String> = buildList {
        val workshop = context.workshop

        if (context.mediumRatio > context.medium.maxUsefulRatio && context.medium != Medium.NONE) {
            add(
                "Proportion de ${context.medium.label.lowercase()} au-dela de la limite utile " +
                    "(${(context.medium.maxUsefulRatio * 100).roundToLong()} %) : le liant ne tient plus le pigment."
            )
        }
        if (workshop.temperatureCelsius < 15) {
            add("Sous 15 degres, l'huile prend tres lentement et peut rester collante plusieurs jours.")
        }
        if (workshop.temperatureCelsius > 28) {
            add("Au-dela de 28 degres, le temps ouvert se reduit fortement : travaillez par petites zones.")
        }
        if (workshop.relativeHumidity > 70) {
            add("Humidite elevee : le sechage tire en longueur, evitez de vernir juste apres.")
        }
        if (context.thickness >= LayerThickness.THICK) {
            add("Couche epaisse : la peau se forme avant le coeur. Attendez le sechage a coeur avant toute manipulation.")
        }
        if (context.medium == Medium.COBALT_DRIER && context.mediumRatio > 0.03) {
            add("Le siccatif au cobalt s'emploie a la goutte : au-dela, la couche craquelle en vieillissant.")
        }
        if (context.dryingClass == DryingClass.VERY_SLOW) {
            add(
                context.dryingClass.typicalPigments +
                    " Ces pigments restent ouverts longtemps : parfait pour les fondus, penible pour les couches suivantes."
            )
        }
        add(context.medium.advice)
    }

    private fun hours(value: Double): Duration = (value * 60).roundToLong().minutes
}
