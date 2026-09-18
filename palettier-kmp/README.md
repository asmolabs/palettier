# Palettier — migration Kotlin Multiplatform

Portage de l'application Java / JavaFX / Spring Boot vers une base Kotlin unique.
**Cibles retenues : Android et Desktop.** Le Web a été écarté, iOS est repoussé — voir
*Décisions*.

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
| domaine — ReadinessWatch, PaletteMix, PaintMatcher | **porté** | 13 tests |
| `core-data` — schéma + catalogue | **porté** | 6 tests sur une vraie base SQLite |
| `core-data` — palettes et projets | **porté** | 8 tests, dont le piège des photos |
| `core-data` — recettes | **porté** | |
| `core-data` — import de la sauvegarde | **porté** | 7 tests sur une vraie archive Java |
| `core-data` — mélanges de palette | **porté** | 2 tests |
| module Koin | **porté** | 3 tests, le graphe se résout |
| `feature-ui` — Aujourd'hui | **porté** | 5 tests |
| `feature-ui` — Projets | **porté** | 4 tests, plus 5 sur `ProjectPlanner` |
| `desktopApp` — client Compose + import | **porté** | lancé et vérifié |
| `feature-ui` — Catalogue | **porté** | 5 tests |
| `core-image` — décodage `expect`/`actual` | **porté** | 5 tests sur JVM |
| `feature-ui` — Pipette | **porté** | 5 tests sur de vraies images |
| `feature-ui` — Palettes | **porté** | 6 tests |
| `feature-ui` — Mélangeur | **porté** | 6 tests |
| `feature-ui` — Séchage, Recettes, Paramètres | à faire | |
| `ai` — Ktor | à faire | |

**≈ 10 100 lignes portées sur 14 639. 144 tests sur JVM, 106 sur Android. Le domaine, la
persistance et le câblage sont faits.** Tout ce qui n'est ni interface ni assistant est porté.

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

## Le pont fonctionne

`BackupImporter` relit une archive produite par l'application Java. L'essai ne travaille
pas sur un fichier fabriqué à la main : l'archive est **sortie du `BackupService` Java**,
catalogue de 680 huiles compris, avec un projet, une couche posée et une photo.

Ce qui traverse, vérifié un par un : les 680 tubes avec leurs pigments, les corrections du
peintre (possession, teinte diluée relevée), les palettes, le projet avec ses zones, ses
six couches dont une variation locale, ses tubes figés, **la pose de la couche avec son
atelier et sa vitesse de séchage**, et la photo rangée à côté du JSON.

Et le dernier essai ferme la boucle entière : `WorkbenchService` calcule l'établi sur les
données restaurées. Archive Java → SQLite → domaine Kotlin, sans adaptateur.

Réimporter la même archive ne duplique rien : le catalogue se met à jour, le travail en
cours est laissé intact. C'est la contrainte `(marque, nom)` qui le garantit, pas du code
de déduplication.

### Ce que l'essai a attrapé

Le premier passage a échoué sur `#C9B9AC` attendu, `#A59CA9` obtenu. Le portage avait
raison : cinq gammes vendent un « Burnt Umber », et l'outil qui a produit l'archive avait
enregistré la teinte sur celui de Gamblin en filtrant sur le nom sans la marque. C'est
exactement le genre d'écart qu'un import « qui a l'air de marcher » laisse passer, et que
seule une vraie archive révèle.

L'essai vérifie maintenant aussi que les quatre homonymes ont gardé la leur : une
correction ne déborde pas sur ses voisins.

## Un `java.text.Normalizer` qui n'existe pas

`PaintMatcher` repliait les accents avec `java.text.Normalizer` — absent du Kotlin commun.
Le remplacement est une table explicite des caractères qui apparaissent réellement dans des
noms d'huiles (français, allemand, italien).

C'est moins puissant qu'une normalisation Unicode, et c'est assumé : une table se lit, donc
on voit ce qu'elle couvre et ce qu'elle ne couvre pas. Un `expect`/`actual` aurait donné
trois implémentations à tenir pour replier une trentaine de caractères.

## L'interface, et ce qui change de nature

Le premier écran est **Aujourd'hui**, celui qu'on a le plus travaillé côté Java.

Ce qui se prouve encore, et qui l'est : le comportement réactif du `WorkbenchViewModel`.
Cinq essais, dont celui qui compte — **le temps qui passe change l'état sans la moindre
écriture en base**. C'est ce que le `combine(observeAll(), ticker)` garantit, et c'est ce
qui fait disparaître tout le rafraîchissement manuel de la version JavaFX : la minuterie, le
bouton *Rafraîchir*, l'écouteur sur la scène, et les appels après chaque écriture. Une
couche cochée dans l'écran Projets rafraîchira celui-ci sans que personne ne relie les deux.

`SharingStarted.WhileSubscribed(5_000)` remplace le `ticker.stop()` que la version JavaFX
plaçait dans un `else` : « ne pas relire la base pour un écran que personne ne regarde »
devient une propriété du cadre au lieu d'une ligne à ne pas oublier.

Ce qui ne se prouve plus : la mise en page, la lisibilité au fond d'un atelier, le confort.
Il faut regarder.

### Les rangs d'affichage ne servent plus à désigner une couche

Côté JavaFX, cocher ou modifier une couche passait par son rang d'affichage, qui différait
du rang stocké pour les variations locales : cliquer « Modifier » sur l'une d'elles levait
une erreur, et il a fallu la corriger.

Ici l'écran montre les couches **telles qu'elles sont rangées** et retrouve leur mélange
par le rôle. C'est plus laborieux qu'un rang commun, et c'est délibéré : les deux ordres ont
divergé une fois déjà, et un mélange attribué à la mauvaise couche ne se voit pas.

### Deux défauts que seule l'exécution a révélés

**`Dispatchers.Main` n'existe pas sur le bureau** sans `kotlinx-coroutines-swing`. Tout
compilait, tous les essais passaient, et l'application s'arrêtait au premier affichage.

**Le schéma était créé à chaque démarrage.** Le premier lancement marchait, le second
échouait sur `table paint already exists` — et l'application ne s'ouvrait plus du tout. Le
défaut ne peut pas se voir sur une base en mémoire, qui est neuve à chaque essai : il a
fallu lancer deux fois. `desktopDriver` lit désormais le pragma `user_version` de SQLite,
crée un schéma neuf, migre un schéma ancien, et laisse tranquille un schéma à jour. C'est ce
qui prend la suite de Liquibase, et deux essais le couvrent maintenant.

## Décisions

**Pas de Web.** L'application affiche « aucune donnée ne part ailleurs » ; la servir
depuis un domaine change la nature de cette promesse, et Wasm imposerait `wa-sqlite` sur
OPFS pour le catalogue et les photos. Écarté.

**iOS repoussé, Android d'abord.** Le périmètre se resserre sur deux cibles. Rien de ce qui
est écrit ne s'y oppose — le domaine n'utilise aucune API de plateforme — mais viser deux
cibles plutôt que trois évite de porter le coût d'iOS (cinterop pour le décodage d'images,
Xcode dans la boucle de vérification) avant d'avoir une application qui marche.

**Le domaine ignore Koin.** Les modules Koin sont déclarés dans `core-data`, pas dans
`core-domain`. Koin est du Kotlin multiplateforme pur, donc il passerait la règle — mais
c'est un framework d'injection, et le domaine n'a pas à savoir qu'on l'injecte. Ce sont des
classes ordinaires qu'on construit à la main, et tous leurs essais le font.

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

## Les deux cibles sont vérifiées

`androidTarget()` est déclaré sur les deux modules, avec `compileSdk 36` et `minSdk 26`.

| Cible | Tests exécutés |
|---|---|
| JVM (Desktop) | **144** |
| Android (debug) | **106** |

Les 75 d'Android sont l'intégralité de `commonTest` du domaine — y compris `MixSearch` sur
le catalogue de 680 huiles. Le moteur tourne donc réellement sur la cible mobile, ce n'était
plus une supposition.

`core-data` n'a pas d'essais Android : les siens s'appuient sur le pilote SQLite JVM. Le
code, lui, compile pour les deux et produit ses AAR.

Le SDK est pointé par `local.properties`, hors du dépôt : Gradle le lit sans qu'on ait à
toucher au shell.

### La vérification des migrations est éteinte

`verifyMigrations` épuise la mémoire de la JVM Gradle, même à 4 Go — et elle n'a rien à
vérifier : le schéma est à sa version 1, il n'existe aucun `.sqm`. Régler le drapeau à
`false` ne suffit pas à empêcher la tâche de s'exécuter, elle est donc désactivée
explicitement.

À rallumer avec la première migration, où elle reprendra tout son sens : elle vérifie alors
qu'appliquer les migrations à l'ancien schéma redonne bien le nouveau. C'est la garantie que
Liquibase ne donnait pas, et on ne veut pas la perdre.

## Construire

```bash
./gradlew :core-domain:jvmTest
```
