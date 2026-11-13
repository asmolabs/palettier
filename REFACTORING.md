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

## Lot 2 : sortir la logique métier des vues — fait

Branche `claude/refactoring-lot-2`. Les vues `workshop.rs` (1 900 lignes), `color.rs`
(1 500), `reference.rs` (1 400) et `miniature.rs` (3 000) sont devenues des modules
`views/workshop/`, `views/color/`, `views/reference/` et `views/miniature/` : un
composant par carte (`ProjectList`, `LayerTable`, `LayerEditor`, `PaletteMatrix`,
`MixerSolverCard`…) et un fichier par outil Miniature. `hud.rs` et `settings.rs` sont
découpés en cartes sur place. 41 fichiers de vue, le plus long fait 562 lignes, aucune
ligne n'est indentée de plus de 40 espaces (10 niveaux).

- La couche `palettier-ui/src/actions/` (dix modules, de `projects` à `settings`) construit
  les `Project`, `PaletteMix`, `Recipe`, `RecipeStep` et `Paint` et les enregistre. Ce
  sont de simples fonctions async qui prennent `&AppServices` et rendent un
  `DomainResult`, sans signal Dioxus, sans boîte de dialogue, sans `spawn`. Chaque
  `spawn(async …)` restant dans une vue se borne à appeler une action puis à poser le
  résultat dans les signaux.
- `AppServices::in_memory()` (tests seulement) monte les services sur
  `Database::in_memory()`, catalogue amorcé. 41 tests d'actions, dont les cas que visait
  le lot 6 : créer un projet acrylique, poser une couche (et l'annuler), l'alerte
  acrylique sur huile (projet et recette), le filtre du mélangeur par système. Plus 13
  tests sur les fonctions extraites des outils Miniature.
- L'état des formulaires reste là où il était (signaux de la vue parente, ou contexte pour
  les outils Miniature et l'écran Projets) : changer d'outil, de projet ou de recette ne
  vide pas ce qui a été saisi.
- Rendu vérifié par le mode capture : les 42 pages HTML (11 écrans en sombre et en clair,
  21 outils) sont identiques avant et après, aux identifiants internes `data-dioxus-id`
  près.

Corrigé au passage : `SeaOrmRecipeRepository::save` insérait toujours, même pour une
recette déjà enregistrée ; « Ajouter l'étape » échouait en silence sur l'unicité du nom.
Elle met maintenant à jour la recette et réécrit ses étapes.

Seule différence visible, et seulement si la base renvoie une erreur : une suppression
qui échoue ne recharge plus la liste et n'annonce plus « supprimé ».

Relevé sans le changer (à trier dans un lot suivant) :
- textes restés en français dans les deux langues (dilutions du lavis, durées du
  congélateur et du mélange-mère, titres de la fiche recette, « Projet « … » supprimé. »),
  ou en anglais dans les deux (« High-Contrast NMM Specular », « + 1 use ») ; le
  sous-titre de Recettes parle d'huile même en acrylique, l'en-tête du banc du mélangeur
  dit « Huile » en acrylique ;
- l'Assistant, avec « Tous mes tubes en stock », génère le plan sur les tubes en stock mais
  enregistre le projet avec tous les tubes du système ;
- la liste des palettes des groupes relevés (Pipette) n'est pas filtrée par système ;
- le HUD retient le projet par sa position dans la liste, et un clic sur sa photo relève
  toujours le centre de l'image ;
- le minuteur du lavis ne se rafraîchit que si autre chose redessine l'écran.

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

## Lot 4 : une seule source pour les libellés — fait

Branche `claude/refactoring-lot-4`.

- Un seul `label(locale)` par type du domaine : les paires `label()` / `label_i18n()`,
  `advice()` / `advice_i18n()`, `typical_pigments`, `meaning`, `verdict`,
  `reachability` sont fusionnées (le texte français est inliné dans la version à
  langue), et `tips()` / `tips_i18n()` disparaissent au profit de `tips_in`. Les
  `*_i18n` restants de l'interface (`Section`…) prennent le même nom.
- Accents rétablis dans le domaine : conseils de séchage, libellés d'épaisseur et
  d'aération, messages du mélangeur et du plan, notes des recettes de démo. Les noms
  des palettes et recettes de démo ne changent pas : le chargement initial les reconnaît
  par leur nom.
- Les conseils de séchage deviennent une énumération `DryingAdvice` qui s'écrit dans
  les deux langues ; ils n'existaient qu'en français. Les tests vérifient la valeur, plus
  la phrase.
- Traduits : fiche recette d'armée (Markdown et HTML), mélanges intermédiaires du guide
  (« Sienne », « Brun foncé », « Noir coloré », « une pointe de »…), durées du
  congélateur, ratio du lavis, consistance, mélange mère, outils NMM, pigments, solvants
  et pinceaux, suppression d'un projet. Le sous-titre de Recettes suit le système, et
  l'en-tête du mélangeur ne dit plus « Huile » en acrylique.
- Le mode capture prend aussi les 11 écrans en anglais. Ce qui y reste en français est
  du contenu enregistré (notes des palettes et étapes des recettes de démo déjà en base,
  pinceaux d'exemple), pas l'interface.

Pas fait : passer à des fichiers de traduction (`fluent`) ; la plupart des textes de
l'interface restent des `locale.tr("…", "…")` en ligne, ce qui suffit à deux langues.

## Lot 5 : styles — fait

Branche `claude/refactoring-lot-5`.

- Les 800 attributs `style: "…"` statiques des vues sont devenus des classes utilitaires,
  une par déclaration CSS : `margin-bottom: 8px` → `u-mb-8`, `font-size: 11.5px` →
  `u-fs-11p5`, `color: var(--accent)` → `u-c-accent`. Elles sont générées en fin de
  `APP_CSS` dans `theme.rs` (251 classes), en `!important` pour garder la priorité
  qu'avait un style en ligne sur les règles de la feuille.
- Restent en ligne les 31 styles qui calculent une valeur (`{…}`) ou qui sont choisis par
  un `if`.
- Rendu vérifié : les 54 pages du mode capture (11 écrans en sombre, en clair et en
  anglais, 21 outils Miniature) ont été comparées avant et après, élément par élément,
  sur 54 propriétés CSS calculées : aucune différence sur 24 901 éléments.

Suite possible : regrouper les combinaisons qui reviennent souvent (par exemple
`u-d-flex u-ai-center u-gap-8`) en classes nommées par leur rôle, au fil des
retouches, plutôt que d'un coup.

## Lot 6 : tests de l'interface et de bout en bout (priorité basse)

Les actions sont testées depuis le lot 2 (dont les quatre cas prioritaires). Reste à
tester les vues elles-mêmes : le mode capture compare déjà le rendu au repos, pas les
clics.

## Corrections ponctuelles à glisser dans le premier lot venu

- Le prompt IA huile annonce « 14 règles » mais n'en liste que 10.
- `SampledPlanService` nomme ses couches en français en dur (« Ombre 1 »,
  « Lumiere 1 ») quelle que soit la langue.
- `default_db_dir()` préfère un `.palettier` dans le dossier courant : lancer l'appli
  depuis un autre dossier peut ouvrir une autre base sans prévenir.

## Ordre conseillé

Lots 1 à 5 faits. Reste le lot 6.
