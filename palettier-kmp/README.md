# Palettier — migration Kotlin Multiplatform

Portage de l'application Java / JavaFX / Spring Boot vers une base Kotlin unique.
**Cibles retenues : Android, iOS, Desktop.** Le Web a été écarté — voir *Décisions*.

L'application Java du dépôt reste la version en service. Ce module vit à côté d'elle et
ne la remplace qu'une fois à parité.

## Où en est le portage

| Partie | État | Vérification |
|---|---|---|
| `color` — Rgb, Lab, CIEDE2000, Kubelka-Munk, Colorant | **porté** | 16 tests, valeurs identiques au Java |
| `paint` — 6 énumérations + `Paint` | **porté** | |
| `drying` — Workshop, DryingContext, DryingTimeService | **porté** | 7 tests, valeurs identiques |
| `mix` — MixSearch, ColorMixService | **porté** | 12 tests sur le vrai catalogue, même pire écart |
| modèle — Palette, Project, Recipe, PaintingPlan | **porté** | |
| `project` — FatOverLean, Substitute, ProgressCheck | **porté** | 14 tests |
| `workbench` — Stage, Bench, WorkbenchService | **porté** | 8 tests |
| `plan` — PlanDryingService | **porté** | 5 tests |
| domaine — ports (`PaintCatalogRepository`, `ProjectRepository`) | **porté** | |
| domaine — ReadinessWatch, PaletteMix, PaintMatcher | à faire | |
| `core-data` — schéma + catalogue | **porté** | 6 tests sur une vraie base SQLite |
| `core-data` — palettes et projets | **porté** | 8 tests, dont le piège des photos |
| `core-data` — recettes, mélanges de palette | à faire | |
| `core-data` — import de la sauvegarde | à faire | |
| `feature-ui` — Compose | à faire | |
| `ai` — Ktor | à faire | |

**≈ 4 100 lignes portées sur 14 639, 76 tests.** La phase 1 est close pour tout ce qui est
du calcul ; la phase 2 tient debout jusqu'aux projets.

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

## Le coût, après parallélisation

| | Java | Kotlin séquentiel | Kotlin parallèle |
|---|---|---|---|
| Recherche sur 680 tubes | 120 ms / cible | 710 ms | **179 ms** |
| Pire écart | 0,85 | 0,8537644933880194 | 0,8537644933880194 |

Mesuré sur la même machine, dans la même session. Le portage séquentiel perdait le
`.parallel()` des flux Java ; `mapChunkedParallel` le rend avec des coroutines sur
`Dispatchers.Default`.

**Le pire écart est identique au chiffre près entre les deux versions Kotlin**, et c'est la
seule chose qui prouve que la parallélisation n'a rien changé au résultat. Ce n'était pas
acquis : deux mélanges à écart strictement égal sont départagés par leur rang de rencontre,
et un tri stable sur une liste réassemblée dans le désordre aurait rendu une autre
proposition. Les morceaux sont donc contigus et remis dans l'ordre, et volontairement plus
nombreux que les cœurs — une boucle triangulaire donne bien plus de travail à ses premiers
indices qu'aux derniers.

`search` et `suggestMixes` sont désormais des `suspend fun`.

## Les services sont des fonctions, plus des beans

Là où Spring injectait un dépôt, le service prend désormais sa donnée en paramètre :
`WorkbenchService.bench(projects, now)`, `SubstituteService.missingAmong(wanted, owned)`,
`ProgressCheckService.compare(project, measured)`. Trois conséquences, toutes bonnes :

- ils se testent sans faux dépôt ni contexte Spring — les tests Java correspondants
  étaient des `@SpringBootTest` à trois secondes de démarrage, les Kotlin sont instantanés ;
- l'instant de référence est un paramètre, donc le temps devient vérifiable ;
- c'est l'appelant qui décide quand relire la base, ce qui est précisément le rôle du
  `Flow` de SQLDelight en phase 2.

`ProgressCheckService` ne décode plus l'image : il reçoit des `DominantColour`. Le décodage
est ce qui n'a rien de commun entre une machine de bureau, un téléphone et un iPhone —
c'est le premier `expect`/`actual` identifié, et le domaine n'a pas à le connaître.

## Ce qui a changé volontairement

- `java.time.Duration` → `kotlin.time.Duration`, sans dépendance externe.
- `DryingProperties` ne vient plus d'`application.yaml` (il n'y en a pas en
  multiplateforme) : ce sont des valeurs par défaut du domaine, surchargeables.
- `Rgb` est une classe à constructeur privé avec une fabrique `invoke` qui **tronque**,
  comme le constructeur compact Java. Une `data class` ordinaire aurait laissé passer des
  composantes hors bornes par `copy()`.
- Les fonctions libres remplacent les classes utilitaires : `deltaE2000(a, b)` au lieu de
  `Colors.deltaE2000(a, b)`, `rgb.toLab()` en extension.

## Le schéma repart à neuf

`core-data` n'essaie pas de convertir la base H2. Les sept changesets Liquibase se soldent :
ils décrivent l'histoire d'un schéma qui n'existera plus. SQLDelight repart d'un `CREATE
TABLE` propre, et les données arrivent par l'import de sauvegarde.

Deux choix de schéma qui ne sont pas neutres :

- **Les pigments ont leur propre table.** Les coller dans une colonne séparée par des
  virgules aurait interdit la recherche par pigment — qui est précisément ce qui donne la
  vitesse de séchage. Ils sont relus en une requête pour tout le catalogue puis regroupés
  en mémoire : une requête par tube ferait sept cents allers-retours pour afficher une liste.
- **`(marque, nom)` est une clé unique.** C'est ainsi que la sauvegarde désigne un tube,
  jamais par un identifiant de base — « un numéro de ligne ne veut rien dire dans une autre
  installation ». La contrainte rend l'import idempotent par construction.

Le fichier reste sous `~/.palettier`, comme du temps de H2 : le peintre n'a pas à savoir
que le moteur a changé, et ses sauvegardes sont déjà rangées à côté.

### Les photos ne peuvent plus disparaître

`ProjectRepository.save` ne touche jamais aux photos, et ce n'est pas un oubli : c'est la
signature qui l'interdit. Côté JPA, un projet lu sans ses photos puis réenregistré les
effaçait — `orphanRemoval` faisait son travail sur une collection vide. Le correctif
consistait à relire le projet dans la transaction, donc à se souvenir de le faire.

Ici les photos ont leurs propres méthodes (`addPhoto`, `removePhoto`) et `save` ne les
voit pas. Le défaut n'est plus évité, il est impossible. Un test le vérifie quand même,
parce qu'une garantie structurelle se casse au premier refactor distrait.

### Les arbres se relisent en quatre requêtes

Projets, tubes figés, zones, couches : quatre requêtes, puis assemblage en mémoire. Une
requête par zone et par couche ferait des centaines d'allers-retours pour afficher une
liste de pièces — et ne se verrait pas sur un jeu d'essai de trois lignes.

À l'enregistrement, les zones sont effacées puis réécrites. Un arbre se remplace en bloc :
réconcilier des rangs qui ont bougé coûte plus cher que de tout reposer.

## Décisions

**Pas de Web.** L'application affiche « aucune donnée ne part ailleurs » ; la servir
depuis un domaine change la nature de cette promesse, et Wasm imposerait `wa-sqlite` sur
OPFS pour le catalogue et les photos. Écarté.

**Le domaine ne dépend de rien qui ne soit du Kotlin multiplateforme pur.** La règle était
d'abord « zéro dépendance ». Elle ne survivait pas à la phase 2 : les ports du domaine
doivent exposer des `Flow`, et `Flow` vit dans `kotlinx-coroutines-core`. Plutôt que de la
contourner en silence, elle est reformulée — coroutines passe, SQLDelight, Compose et tout
framework ne passent pas. L'intention est intacte : pas de plateforme, pas de persistance,
pas d'interface. Et la recherche de mélange y gagne sa parallélisation.

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
