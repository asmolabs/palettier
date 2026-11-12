# Plan de refactoring

État des lieux après la séparation huile / acrylique (branche `claude/acrylique-ou-huile`).
Le domaine et la persistance sont sains ; la dette est concentrée dans l'onglet Miniature
et dans les vues. Chaque lot ci-dessous se livre seul, avec la suite de tests au vert
(100 tests aujourd'hui), et ne change pas ce que voit le peintre sauf mention contraire.

## Ce qui tient et qu'on ne touche pas

- Le découpage en crates (domaine sans interface, `#![forbid(unsafe_code)]` partout).
- Kubelka-Munk, le catalogue chargé depuis les CSV et la table des pigments.
- Les migrations SQLite et la compatibilité avec les sauvegardes de la version Java,
  vérifiée par `real_java_backup_imports_and_roundtrips_intact`.

## Lot 1 : vider `miniature.rs` de ses doublons — fait

Branche `claude/refactoring-lot-1`. `miniature.rs` (2 500 lignes) est devenu un module
`miniature/` en six fichiers par thème (`colour`, `effects`, `application`, `curing`,
`batch`, `studio`), réexportés tels quels : les imports existants n'ont pas changé.

| Doublon | Devenu |
|---|---|
| `MINIATURE_ACRYLICS`, `MiniatureBrand`, `search_miniature_acrylics`, `find_closest_acrylic` | supprimés ; le convertisseur et la recherche ⌘K lisent les acryliques du catalogue (fichiers `10-…14-*.csv`) |
| `OilMedium` | supprimé ; la carte des médiums lit `Medium::for_system(Oil)`, avec le facteur de séchage et la limite utile du modèle |
| `OilConsistencyTechnique` | remplacé par `Consistency::of(Technique)` (classement dérivé, pas une seconde liste) et `thinner_drops_per_part` calculé depuis la proportion de médium de la technique |
| `HeatBoxCalculator` (« Arrhenius » sans source) | garde les seuils de sécurité des plastiques ; l'accélération vient de `DryingTimeService` (×4 à 40 °C) |
| `LivePaletteFreshnessTracker`, `FreezerStorageSimulator` (seuils inventés) | lus sur le séchage d'une noisette d'huile sur la palette ; le congélateur donne ~29 jours, l'ancienne valeur codée en dur était 28 |

Corrigé au passage : les harmonies et l'aperçu de glacis choisissaient leur « teinte
d'huile » dans la liste des acryliques ; ils puisent maintenant dans la palette active.

Reste hors lot : les autres outils (NMM, OSL, weathering, recettes d'armée) gardent des
teintes et recettes codées en dur. Ce ne sont pas des doublons du domaine, mais leurs
recettes citent des tubes en texte libre plutôt que des fiches du catalogue.

## Lot 2 : sortir la logique métier des vues (priorité haute)

`workshop.rs` (1 800 lignes, jusqu'à 108 espaces d'indentation), `color.rs` (1 500) et
`views/miniature.rs` (2 760) construisent des `Project`, des `PaletteMix` et des
`RecipeStep` directement dans des `onclick`, avec 47 `spawn(async …)` dispersés.

- Une couche d'actions dans `palettier-ui/src/actions/` (`create_project`,
  `place_mix`, `add_recipe_step`, …) : de simples fonctions async qui prennent
  `AppServices` et rendent un `DomainResult`. Les vues ne font plus qu'appeler.
- Un composant par carte : `ProjectList`, `LayerTable`, `LayerEditor`,
  `PaletteMixCard`, et un sous-module par onglet Miniature.
- Objectif : aucun fichier de vue au-delà de 600 lignes, pas plus de 10 niveaux
  d'imbrication.

## Lot 3 : typer la technique d'une couche — fait

Branche `claude/refactoring-lot-3`.

- `ProjectLayer.technique` et `PlanLayer.technique` sont des `Technique`, plus des
  chaînes. Un libellé inconnu (sortie d'IA, ancienne saisie) n'est plus remplacé en
  silence : la couche prend la technique par défaut du système de la pièce et le
  libellé est conservé en tête de la note (« Technique proposée : … »).
- `Technique::parse` reconnaît le code, le libellé français ou anglais, en ignorant la
  casse et les accents. Les libellés ont donc retrouvé leurs accents (« Jus à l'huile »,
  « Liserés d'arêtes »…) sans casser la lecture des anciennes données.
- Migration `m0003_layer_technique_code` : `project_layer.technique` passe des
  libellés aux codes. Elle convertit aussi les couches orphelines (zone supprimée alors
  que les clés étrangères étaient coupées). Puis `m0004_orphan_layer_cleanup` supprime
  ces couches orphelines, que rien n'affiche. Essayées sur une copie d'une vraie base :
  32 couches converties, les 24 orphelines supprimées, les 8 couches des 3 projets
  intactes.
- La sauvegarde écrit le libellé français sans accents (`legacy_label`), la forme que
  la version Java relit ; à l'import, elle lit tout libellé connu et garde les autres
  dans la note.
- Un nouveau projet ou une nouvelle zone démarre avec des techniques adaptées au
  système de la pièce (en acrylique : lavis d'ombrage, couches, aplat, liserés).

## Lot 4 : une seule source pour les libellés (priorité moyenne)

21 paires `label()` / `label_i18n()` répètent les mêmes `match`. Le domaine écrit sans
accents (« sechage », « Pate »), l'interface avec.

- Un seul `label(locale)` par type, `Display` délégué à `label(Locale::Fr)`.
- Rétablir les accents dans le domaine. Les tests qui cherchent un texte
  (`contains("limite utile")`, `contains("15 degres")`) devront tester une variante
  d'enum ou une clé, pas une phrase.
- Plus tard, si l'anglais grandit : passer aux fichiers de traduction (`fluent`).

## Lot 5 : styles (priorité basse)

786 attributs `style: "…"` inline. Les rassembler en classes dans `theme.rs` (dont
`card-subtle`, `row-between`, et les couleurs huile / acrylique déjà ajoutées) rend
le thème clair / sombre cohérent et allège les vues avant le lot 2.

## Lot 6 : tests de l'interface et de bout en bout (priorité basse)

Aujourd'hui, rien ne teste les vues. Une fois le lot 2 fait, les actions se testent
comme le domaine, sur une base en mémoire (`Database::in_memory()`). À couvrir en
premier : créer un projet acrylique, poser une couche, voir l'alerte acrylique sur
huile, et le filtre du mélangeur par système.

## Corrections ponctuelles à glisser dans le premier lot venu

- Le prompt IA huile annonce « 14 règles » mais n'en liste que 10.
- `SampledPlanService` nomme ses couches en français en dur (« Ombre 1 »,
  « Lumiere 1 ») quelle que soit la langue.
- `default_db_dir()` préfère un `.palettier` dans le dossier courant : lancer l'appli
  depuis un autre dossier peut ouvrir une autre base sans prévenir.

## Ordre conseillé

Lots 1 et 3 faits. Ensuite 2 → 4 → 5 → 6.
