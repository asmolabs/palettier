# Palettier

Assistant de peinture à l'huile pour figurines, bustes et maquettes.

L'huile pose au peintre des problèmes que l'acrylique ne pose pas : un mélange se
comporte de façon soustractive, le séchage se compte en jours et dépend du pigment
autant que de la dilution, et une recette n'est pas une liste de couches mais un
planning. Le logiciel répond à ces trois questions.

| Section | Ce qu'elle résout |
|---|---|
| **Catalogue** | 439 huiles de 8 gammes ; laquelle se rapproche le plus d'une teinte visée (écart CIEDE2000) |
| **Palettes** | Une sélection nommée de tubes par sujet, qui sert ensuite de filtre de recherche |
| **Pipette** | Relever une teinte sur une photo (ou la saisir en hexa) et obtenir le mélange à faire |
| **Mélangeur** | Quelle couleur donne ce mélange — et, à l'inverse, quel mélange donne cette couleur |
| **Séchage** | Quand puis-je reprendre la pièce : temps ouvert, recouvrable, sec à cœur, vernissable |
| **Recettes** | Combien de séances représente vraiment cette recette |

## Stack

- Java 25 (LTS)
- Spring Boot 4.1.1 — injection, configuration, Spring Data JPA
- JavaFX 25.0.4 (LTS) — client lourd, interface construite en Java (pas de FXML)
- H2 en fichier local, sous `~/.palettier/`

## Démarrer

```bash
mvn install -DskipTests   # publie palettier-core dans le dépôt local
mvn -pl palettier-ui javafx:run
```

Ou, après un `mvn package`, le jar exécutable :

```bash
java -jar palettier-ui/target/palettier-ui-0.1.0-SNAPSHOT.jar
```

Tests :

```bash
mvn test
```

## Organisation

```
palettier-core/   domaine, colorimétrie, services — aucune dépendance à JavaFX
palettier-ui/     client JavaFX piloté par Spring
```

Le découpage n'est pas décoratif : `palettier-core` ne connaît pas l'interface, ce qui
permet de lui ajouter plus tard une API REST ou un client mobile sans rien déplacer.
Le module expose une seule porte d'entrée, `CoreConfiguration`, que l'application hôte importe.

### Points d'extension

- **Une section** = un bean qui implémente `AppView` (`title`, `subtitle`, `create`,
  `order`). `MainWindow` collecte tous ceux présents dans le contexte, construit la
  navigation et n'instancie le contenu qu'à la première ouverture. Rien d'autre à déclarer.
- **Le catalogue de départ** est dans `CatalogSeeder`, chargé au premier démarrage
  seulement.
- **Le modèle de séchage** est paramétré dans `application.yaml` sous `palettier.drying` :
  si votre atelier vous donne systématiquement des durées différentes, ajustez-y les
  facteurs plutôt que le code.

## Le catalogue

Huit gammes, **uniquement à l'huile**, aucune acrylique. Un test (`CatalogDataTest`)
vérifie que la liste des marques est exactement celle-ci — une gamme ajoutée par erreur
fait échouer le build.

| Gamme | Fiches | Couverture |
|---|---|---|
| Winsor & Newton Artists' Oil | **120** | quasi complète (la gamme annonce « plus de 135 », métalliques exclues) |
| Winsor & Newton Winton | 46 | quasi complète |
| Schmincke Norma Professional | 58 | partielle (~84) |
| Schmincke Norma Blue (diluable à l'eau) | 28 | **vérifiée auprès du fabricant**, 28 sur 48 |
| Gamblin Artist's Oil Colors | 64 | partielle (~110) |
| Williamsburg Handmade Oil | 59 | partielle (~170) |
| Scale75 Floww | 18 | **18 sur 24** — le coffret Scenery manque |
| Abteilung 502 (AK) | 46 | partielle (~90) |

### La table des pigments, pièce centrale

`catalog/pigments.csv` décrit 82 pigments du Colour Index. C'est délibéré : **la vitesse
de séchage et le pouvoir colorant ne sont pas des choix de marque, ce sont des propriétés
du pigment.** Une terre d'ombre (PBr7) sèche vite chez tout le monde, un cadmium (PY35)
sèche lentement chez tout le monde. Les renseigner une fois évite de les deviner tube par
tube — et c'est ce qui permet d'ajouter une gamme entière sans rien inventer.

Chaque fiche de gamme ne porte donc que ce que les fabricants publient réellement :
nom, référence, pigments, opacité, teinte.

### Calibrer un tube d'après vos écouvillons

Dans le **Catalogue**, sélectionnez une ligne : la carte « Calibrer le tube sélectionné »
montre son ton de masse et sa teinte coupée de blanc, et permet de les remplacer par une
mesure.

Le protocole : peignez un écouvillon de couleur pure et un autre d'une part de couleur pour
neuf de blanc de titane, photographiez-les sous lumière neutre, relevez-les dans la Pipette
(qui corrige la dominante), revenez ici et attribuez-les. C'est la seconde mesure qui fait
passer le tube au modèle à deux constantes.

Le découpage est délibéré : **la Pipette mesure, le Catalogue attribue.** C'est dans le
tableau qu'on voit ce qui manque, et c'est là qu'on dispose déjà de la recherche, des
filtres et de la colonne « Coupé de blanc ». Un unique objet partagé (`SampledColor`) porte
la dernière teinte relevée d'un écran à l'autre, sans qu'aucun des deux ait à connaître
l'autre.

Une valeur venue de votre peinture sous votre lumière vaut mieux que n'importe quel
nuancier imprimé — et vos mesures ne sont jamais écrasées par une mise à jour du catalogue.

### Ajouter ou corriger un tube

Une ligne dans le fichier de la gamme, sous `palettier-core/src/main/resources/catalog/brands/` :

```csv
brand;name;code;pigments;opacity;hex
Gamblin;Transparent Earth Red;1970;PR101;TRANSPARENT;#8A2E15
```

Seules `brand`, `name` et `pigments` sont obligatoires : l'opacité et la teinte sont
déduites des pigments si les colonnes sont vides, et le tube est alors marqué comme
approximatif. Une marque supplémentaire = un fichier supplémentaire, sans recompilation
du code métier.

Le catalogue n'est chargé **que si la base est vide**, donc vos corrections dans
l'application ne sont jamais écrasées. Pour repartir des fichiers : supprimez
`~/.palettier/`.

### Ce qui est fiable et ce qui ne l'est pas

- **Vérifié par test** : la divergence du noir d'ivoire (chaud au tube, froid en mélange)
  est reproduite par le modèle et assertée.
- **Fiable** : les codes pigments des gammes beaux-arts (W&N, Gamblin, Williamsburg,
  Schmincke) — ces fabricants les publient.
- **Déduit, à corriger sur vos tubes** : les pigments de **Scale75 Floww** et
  **Abteilung 502**. Ces gammes maquette ne publient pas leurs codes Colour Index ; ils
  sont inférés du nom de la couleur. Ils pilotent le calcul du séchage, donc les durées
  annoncées pour ces deux gammes sont approximatives tant qu'ils ne sont pas corrigés.
  L'avertissement est répété en tête de chaque fichier concerné.
- **Estimé, à corriger** : la colonne `tint` de `pigments.csv` (28 pigments), qui pilote le
  modèle à deux constantes. Elle n'est remplie que là où la divergence ton de masse / teinte
  diluée est franche et connue — surtout les pigments à masse très sombre et teinte colorée
  (noirs, bleu de Prusse, phtalos, dioxazine, alizarine). Ailleurs elle reste vide, et le
  comportement est celui d'avant.
- **Approximatif par nature** : toutes les valeurs hexadécimales. Un même code PBr7
  couvre des teintes très différentes selon le gisement. La façon propre de les fiabiliser
  est de photographier ses propres écouvillons sous lumière neutre.

## Les palettes

Sur une figurine on ne travaille jamais avec le catalogue entier, mais avec six à douze
tubes choisis ensemble. Une palette est cette sélection, nommée et rangée par sujet.

Son intérêt n'est pas seulement l'organisation : elle sert de **filtre de recherche**
dans le mélangeur. Chercher « comment obtenir cette couleur » dans 398 tubes donne une
réponse théorique, souvent avec des tubes qu'on ne possède pas ; la même recherche
limitée à une palette donne une recette réalisable tout de suite.

Trois palettes sont livrées, dont la **palette Zorn** (Winsor & Newton) : blanc de titane,
ocre jaune, rouge de cadmium clair, noir d'ivoire. Les palettes de référence sont ajoutées
une par une, par nom : une nouvelle palette livrée dans une version ultérieure arrive sans
toucher à celles que vous avez composées.

La palette affiche aussi ce qu'elle impose : la classe de séchage de son tube le plus
lent — c'est elle qui commande le planning d'une séance — et son nombre de pigments
distincts, avec un avertissement au-delà de six, seuil à partir duquel les mélanges à
trois tubes virent au gris.

## La pipette

Chemin inverse du mélangeur : on part du résultat voulu — vu sur une photo de référence,
une pièce existante, un nuancier — et on remonte aux tubes.

- **Ouvrir une photo** (ou la glisser-déposer). Elle s'affiche **entière par défaut**, et
  se réajuste quand la fenêtre change de taille. Zoom de 5 % à 1600 % : boutons, molette
  touche commande enfoncée, ou pincement sur pavé tactile ; « Ajuster » et « 100 % » en un
  clic. Au-delà de la taille réelle le lissage est coupé, pour voir les pixels plutôt qu'un
  flou d'interpolation.
- **Clic ou cliquer-glisser** pour relever une teinte, à n'importe quel zoom.
- **Rayon d'échantillonnage** : la moyenne d'un carré de pixels vaut toujours mieux qu'un
  pixel isolé, qui porte le bruit du capteur. La moyenne est calculée **en lumière linéaire**,
  pas sur les valeurs sRGB : moyenner du sRGB directement assombrit le résultat (moyenner
  noir et blanc donne `#BCBCBC`, pas `#808080` — c'est vérifié par un test).
- **Correction de dominante** : une photo prise sous lampe de bureau tire au jaune. Désignez
  un point censé être gris ou blanc neutre, et la dominante est retirée de tous les relevés
  suivants. C'est la balance des blancs réduite au strict nécessaire.
- **Saisie hexadécimale** directe, synchronisée avec la pastille, si vous n'avez pas de photo.
- **La teinte relevée est publiée vers le Catalogue**, où elle peut être attribuée à un
  tube (voir ci-dessous). La Pipette mesure, le Catalogue attribue.
- **Recherche dans une palette**, en stock, ou dans tout le catalogue. Chaque proposition
  affiche la teinte visée et le mélange obtenu **côte à côte**, les parts à doser, et l'écart
  traduit en langage de peintre.

## L'interface

Codes d'une application web plutôt que d'un client lourd : **barre latérale de
navigation** (pas d'onglets), en-tête de page avec titre et sous-titre, contenu en
**cartes** posées sur un fond uni, bordures fines, coins arrondis, étiquettes arrondies
pour l'opacité et la vitesse de séchage. Aucun relief, aucun dégradé de bouton système.

Le thème est sombre et chaud, accents cuivre. Ce n'est pas qu'un parti pris : l'écran
sert ici à juger des pastilles de couleur, et un fond clair les fait paraître plus
sombres et plus saturées qu'elles ne sont.

Tout tient dans `palettier-ui/src/main/resources/be/asmolabs/palettier/ui/app.css`, dont
la palette est déclarée en variables en tête de fichier (`-pal-canvas`, `-pal-accent`,
`-pal-border`…) : changer l'ambiance revient à changer une douzaine de lignes.

Deux composants portent l'essentiel de la mise en page : `Card` (un bloc titré, qui
remplace `TitledPane`) et `Pill` (l'étiquette arrondie des tableaux).

## L'assistant

Module `palettier-ai`, **inactif par défaut**. Sans `PALETTIER_AI_PROVIDER`, aucune clé
n'est lue, aucun appel n'est fait, et l'application se comporte exactement comme si le
module n'existait pas — un test le vérifie.

Trois moteurs au choix :

| Moteur | Variables | Remarque |
|---|---|---|
| **Gemini** (palier gratuit AI Studio) | `PALETTIER_AI_PROVIDER=google-genai`, `GEMINI_API_KEY` | le palier gratuit autorise Google à réutiliser les contenus envoyés |
| **Ollama** (local) | `PALETTIER_AI_PROVIDER=ollama`, modèle avec capacité `vision` | rien ne quitte le poste |
| **Compatible OpenAI** | `PALETTIER_AI_PROVIDER=openai`, `OPENAI_BASE_URL`, `OPENAI_API_KEY` | local (LM Studio, llama.cpp, vLLM) ou distant |

### La séparation des rôles

**Le modèle choisit quelles couleurs viser ; l'application calcule comment les atteindre.**

Le modèle découpe le sujet en zones et propose, pour chacune, une teinte de base, une
ombre et une lumière — en hexadécimal, jamais en mots. Il n'écrit **aucun dosage et
aucune durée**. Pour chaque couleur visée, `MixSearch` calcule ensuite le mélange qui
l'approche le mieux avec les tubes de la palette choisie, et l'écart affiché est une
mesure CIEDE2000, pas une affirmation.

L'écran montre côte à côte la couleur visée et celle réellement obtenue : quand la
palette ne suit pas, cela se voit.

### Photos

Une photo de la pièce, et autant de références que voulu. Les zones sont alors déduites
de ce qui est visible, pas de ce qu'un tel sujet comporte en général.

Les images sont **réduites à 1024 pixels et réencodées en JPEG** par le service avant
l'envoi — la règle est imposée là et non laissée à l'appelant. Elles servent à
reconnaître les zones et juger l'avancement, **jamais à relever une couleur** : une photo
est compressée et prise sous une lumière quelconque. Pour mesurer, la pipette travaille
en local sur votre fichier d'origine.

## Les modèles de calcul

### Mélange des couleurs

Un mélange n'est pas une moyenne RGB. Deux huiles qui se mélangent se comportent comme
des milieux diffusants : c'est le modèle de **Kubelka-Munk** qui est implémenté, où
absorption `K` et diffusion `S` sont additives. C'est ce qui fait que bleu + jaune donne
du vert et non du gris.

**Deux constantes, pas une** (`Colorant`). Le modèle à constante unique suppose que tous
les pigments diffusent la lumière de la même façon et ne retient que le rapport `K/S`. Il
ne peut alors pas représenter un cas pourtant banal : le noir d'ivoire sort du tube
franchement chaud (`#221F1C`) et donne pourtant des gris **froids** une fois coupé de blanc
(`#8E9094`). Une seule valeur par tube ne peut pas porter ces deux comportements.

Avec `K` et `S` séparés, les deux coexistent. Le mécanisme : au ton de masse la couleur
suit `K/S`, en teinte diluée la diffusion du blanc domine tellement que la couleur suit `K`
seul. Si la diffusion du pigment n'est pas la même sur tous les canaux, les deux couleurs
divergent. Pour le noir d'ivoire, la résolution donne `K = 9,13 / 8,58 / 7,59` et
`S = 0,30 / 0,24 / 0,18` : plus d'absorption dans le rouge (donc teinte froide) mais aussi
plus de diffusion (donc ton de masse chaud).

Les constantes sont ajustées, canal par canal, sur **deux couleurs observées** — le ton de
masse et la teinte coupée de blanc, une part pour neuf. Le système se résout en forme
close ; si la teinte fournie est incohérente avec le ton de masse, le canal retombe sur le
modèle à constante unique plutôt que de produire des constantes négatives.

**Dégradation propre.** Un tube sans teinte diluée renseignée donne `S = 1` sur les trois
canaux, ce qui ramène *exactement* au modèle à constante unique — un test vérifie l'égalité
au code hexadécimal près. Le passage à deux constantes n'a donc rien cassé de ce qui
marchait : il n'améliore que là où la donnée existe. Aujourd'hui **312 huiles sur 448**.

Ce n'est pas une mesure au spectrophotomètre sur trente longueurs d'onde : c'est un
ajustement sur deux points, qui les reproduit fidèlement et interpole entre eux.

### Pourquoi seulement 29 pigments sur 83

Les 7 blancs n'ont pas de teinte diluée : ils **sont** la référence. Restent 47 pigments
sans mesure, et c'est un choix, pas un oubli.

La divergence ton de masse / teinte diluée est d'autant plus forte que le pigment est
sombre et peu diffusant. Quand `K/S` est grand, le ton de masse suit le **rapport** `K/S`
alors que la teinte suit `K` seul — une faible variation de `S` entre canaux suffit à
écarter les deux. Les cas spectaculaires sont donc toujours les mêmes : noirs, bleu de
Prusse, phtalos, dioxazine, alizarine. Ce sont exactement ceux qui sont renseignés.

Pour les organiques de valeur moyenne — jaunes arylides, oranges benzimidazolone, rouges
naphtol — la teinte réelle est proche de ce que le modèle à constante unique prédit déjà.
Le gain marginal y est faible, et une valeur inventée y ferait plus de mal que de bien :
une teinte fausse fausse activement tous les mélanges du tube, alors qu'une teinte absente
laisse le modèle précédent, connu et prévisible.

**La bonne façon de compléter n'est pas que j'estime davantage, c'est de mesurer.** L'onglet
Pipette le permet (voir ci-dessous).

Chaque huile est pondérée par sa dose **et** par son pouvoir colorant : une pointe de
bleu de Prusse dans un blanc de titane pèse beaucoup plus que son volume, et l'interface
le signale.

### La recherche inverse

« Comment obtenir cette couleur ? » — `MixSearch`. Quatre idées, dont la première est la
plus importante :

**1. Ne proposer que des dosages réalisables.** Un optimum continu du type « 0,371 part
contre 0,629 » n'a aucun sens au bout d'un pinceau. La recherche n'explore que des
rapports d'entiers simples (jusqu'à 8 parts, plus les ajouts infimes 1:10 à 1:30 — une
pointe de bleu de Prusse dans un blanc, c'est du 1:20). **L'écart annoncé est donc celui du
mélange qu'on peut réellement faire**, pas celui d'un optimum inatteignable.

**2. Deux passes.** Toutes les paires sont évaluées à cinq dosages de repérage ; seules les
60 meilleures sont reprises sur la grille complète. On paie le prix fort uniquement là où
cela peut changer le classement.

**3. Trois tubes sur un vivier restreint.** Les explorer tous serait hors de portée
(10 millions de triplets). Mais un bon mélange se compose presque toujours de tubes déjà
proches de la cible, **plus un tube « structurel »** à fort pouvoir colorant — un blanc, un
noir — qui sert à monter ou descendre la valeur. Le vivier réunit ces deux familles ; une
sélection fondée sur la seule proximité manquerait la seconde.

**4. À écart imperceptible, le mélange le plus simple gagne.** Sous 0,5 ΔE l'œil ne fait
plus la différence : c'est alors la simplicité qui départage. Sans cette règle la recherche
répond « quatre parts de terre d'ombre brûlée W&N, cinq de Schmincke et trois de Norma
Blue » là où un seul tube suffisait. Les propositions aboutissant à la même teinte sont
aussi regroupées — le catalogue contient la même couleur chez plusieurs fabricants.

Le tout tourne en parallèle. Sur le catalogue complet : **~50 ms par cible, pire écart 0,85**
sur une batterie de dix teintes réelles (carnations, terrains, blindages). La version
précédente — paires seulement, grille fixe de 1:9 à 9:1 — mettait 1 060 ms.

### Séchage

Le temps de référence vient de la **classe de séchage du pigment** — à l'huile, c'est le
pigment qui commande, pas la marque. On lui applique ensuite l'effet du médium, de
l'épaisseur, de la température (la vitesse d'oxydation double environ tous les 10 °C), de
l'humidité et de la ventilation.

Cinq jalons sont produits : temps ouvert, sec au toucher, recouvrable, sec à cœur,
polymérisation complète. Dans une recette, une technique qui exige un support ferme
(`Technique.requiresCuredBase`) attend le séchage à cœur de la couche précédente ; les
autres se contentent du délai de recouvrement.

## Limites assumées

- Les valeurs hexadécimales du catalogue sont des **approximations de la teinte sortie de
  tube**, destinées à l'aperçu et au calcul. Elles ne remplacent pas un nuancier physique ;
  la façon propre de les fiabiliser est de photographier ses propres écouvillons sous
  lumière neutre et de corriger les valeurs dans l'onglet Catalogue.
- Les durées de séchage sont des ordres de grandeur pour planifier une séance, pas une
  mesure. Sur la pièce, la vérification reste tactile.
- Le catalogue n'est pas encore éditable depuis l'interface (lecture et recherche
  seulement) : `PaintCatalogService.save` et `delete` existent, il manque le formulaire.

## Pistes suivantes

- Compléter les gammes : les 6 Scale75 Scenery, puis les fiches manquantes des gammes
  beaux-arts (le format CSV est fait pour ça)
- Corriger les pigments inférés de Scale75 et Abteilung 502 d'après les tubes
- Édition du catalogue depuis l'interface, ou calibration depuis une photo d'écouvillons
- Minuteurs de séchage avec notification système, adossés aux jalons déjà calculés
- Recettes modifiables depuis l'interface, avec photos d'étape
- Passage de deux à trois tubes dans la recherche inverse de mélange
