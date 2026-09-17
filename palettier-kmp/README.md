# Palettier — migration Kotlin Multiplatform

Portage de l'application Java / JavaFX / Spring Boot vers une base Kotlin unique.
**Cibles retenues : Android, iOS, Desktop.** Le Web a été écarté — voir *Décisions*.

L'application Java du dépôt reste la version en service. Ce module vit à côté d'elle et
ne la remplace qu'une fois à parité.

## Où en est le portage

| Partie | État | Vérification |
|---|---|---|
| `color` — Rgb, Lab, CIEDE2000, Kubelka-Munk, Colorant | **porté** | 16 tests, valeurs identiques au Java |
| `paint` — DryingClass, Medium, Opacity, LayerThickness, Ventilation, Technique | **porté** | couvert indirectement |
| `drying` — Workshop, DryingContext, DryingEstimate, DryingTimeService | **porté** | 7 tests, valeurs identiques |
| `mix` — MixSearch, ColorMixService | **porté** | 12 tests sur le vrai catalogue, même pire écart |
| modèle — `Paint` | **porté** | |
| modèle — Palette, Project, Recipe | à faire | |
| `core-data` — SQLDelight | à faire | |
| `feature-ui` — Compose | à faire | |
| `ai` — Ktor | à faire | |

**≈ 1 700 lignes portées sur 14 639.** C'est peu en volume et beaucoup en risque retiré :
la colorimétrie, le séchage et la recherche de mélange sont ce que tout le reste suppose
juste.

## La méthode, et pourquoi elle tient

Chaque test Java est traduit **avec ses valeurs attendues inchangées**. `#BCBCBC` pour la
moyenne en lumière linéaire, l'écart nul d'une couleur avec elle-même, le doublement du
séchage à dix degrés de moins : si le portage déplaçait une virgule, ces tests le diraient.

C'est le seul contrôle qui vaille sur du code numérique. Une relecture ne voit pas qu'un
`Math.toRadians` a disparu dans une conversion.

## Le catalogue sert d'oracle

`Catalogue.kt` (694 lignes, dans `commonTest`) est **généré** depuis le `CatalogLoader`
Java, pas ressaisi. Les tests de portage jugent donc sur exactement les 680 huiles des
tests d'origine, avec leurs pigments, leurs teintes diluées relevées et leurs pouvoirs
colorants.

Le résultat qui compte :

| | Java | Kotlin |
|---|---|---|
| Pire écart sur 10 teintes réelles | 0,85 | 0,8537644933880194 |

Le pire écart est la sortie de toute la chaîne d'optimisation — passe grossière, grille de
dosages, vivier, classement par palier perceptuel. Qu'il tombe au même centième signifie
que le portage est fidèle jusque dans ses arbitrages.

## Ce qui régresse : 5,8× plus lent

| | Java | Kotlin |
|---|---|---|
| Recherche sur 680 tubes | 120 ms / cible | 710 ms / cible |

Mesuré sur la même machine, dans la même session. La cause est connue et unique :
`MixSearch.pairs()` et `combinations()` utilisaient `.parallel()` sur des flux Java.
`commonMain` n'a pas d'équivalent — le portage est séquentiel.

Ce n'est pas un défaut de traduction, c'est une décision en attente. La lever demande
`kotlinx-coroutines-core` dans `core-domain` et une `suspend fun search`, ce qui contredit
la règle « le domaine ne dépend de rien » et change tous les appelants. À trancher avant
`core-data`, pas après.

Nuance utile : le cas à 680 tubes est celui du Catalogue et de la Pipette. Un plan de
projet cherche dans une palette de six à douze tubes, où le coût est sans commune mesure.

## Ce qui a changé volontairement

- `java.time.Duration` → `kotlin.time.Duration`, sans dépendance externe.
- `DryingProperties` ne vient plus d'`application.yaml` (il n'y en a pas en
  multiplateforme) : ce sont des valeurs par défaut du domaine, surchargeables.
- `Rgb` est une classe à constructeur privé avec une fabrique `invoke` qui **tronque**,
  comme le constructeur compact Java. Une `data class` ordinaire aurait laissé passer des
  composantes hors bornes par `copy()`.
- Les fonctions libres remplacent les classes utilitaires : `deltaE2000(a, b)` au lieu de
  `Colors.deltaE2000(a, b)`, `rgb.toLab()` en extension.

## Décisions

**Pas de Web.** L'application affiche « aucune donnée ne part ailleurs » ; la servir
depuis un domaine change la nature de cette promesse, et Wasm imposerait `wa-sqlite` sur
OPFS pour le catalogue et les photos. Écarté.

**Le domaine ne dépend de rien.** `core-domain/build.gradle.kts` n'a aucune dépendance
`commonMain`. Ce n'est pas une élégance : c'est ce qui garantit que le moteur compile sur
iOS sans qu'on ait à le découvrir six mois plus tard.

**La migration des données passe par la sauvegarde.** `BackupService` exporte déjà tout
dans un ZIP JSON indépendant des entités. L'ancienne application exporte, la nouvelle
importera. Aucun convertisseur H2 → SQLite à écrire, et les 7 changesets Liquibase ne se
rejouent pas.

## Limites de vérification sur cette machine

Ni SDK Android ni Xcode installés. Seule la cible `jvm()` est déclarée et testée. Les
cibles Android et iOS sont commentées dans `core-domain/build.gradle.kts` avec la ligne
exacte à décommenter : déclarer une cible qu'on ne peut pas compiler donnerait une
illusion de vérification.

## Construire

```bash
./gradlew :core-domain:jvmTest
```
