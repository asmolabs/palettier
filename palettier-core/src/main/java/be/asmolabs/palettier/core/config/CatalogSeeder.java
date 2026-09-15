package be.asmolabs.palettier.core.config;

import be.asmolabs.palettier.core.catalog.CatalogLoader;
import be.asmolabs.palettier.core.domain.LayerThickness;
import be.asmolabs.palettier.core.domain.Medium;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.domain.Recipe;
import be.asmolabs.palettier.core.domain.RecipeStep;
import be.asmolabs.palettier.core.domain.Technique;
import be.asmolabs.palettier.core.repository.OilPaintRepository;
import be.asmolabs.palettier.core.repository.PaletteRepository;
import be.asmolabs.palettier.core.repository.RecipeRepository;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.ApplicationArguments;
import org.springframework.boot.ApplicationRunner;
import org.springframework.stereotype.Component;
import org.springframework.transaction.annotation.Transactional;

/**
 * Remplit la base au premier demarrage : le catalogue vient des fichiers de
 * {@code catalog/brands/}, les recettes d'exemple sont definies ici.
 *
 * <p>Le catalogue n'est charge que si la base est vide. Les corrections apportees par
 * le peintre a ses propres fiches ne sont donc jamais ecrasees par une mise a jour des
 * fichiers de gamme ; pour repartir des fichiers, supprimez
 * {@code ~/.palettier/}.</p>
 */
@Component
class CatalogSeeder implements ApplicationRunner {

    private static final Logger log = LoggerFactory.getLogger(CatalogSeeder.class);

    private final OilPaintRepository paints;
    private final RecipeRepository recipes;
    private final PaletteRepository palettes;
    private final CatalogLoader catalogLoader;

    CatalogSeeder(OilPaintRepository paints, RecipeRepository recipes,
                  PaletteRepository palettes, CatalogLoader catalogLoader) {
        this.paints = paints;
        this.recipes = recipes;
        this.palettes = palettes;
        this.catalogLoader = catalogLoader;
    }

    @Override
    @Transactional
    public void run(ApplicationArguments args) {
        if (paints.count() == 0) {
            paints.saveAll(catalogLoader.load());
            log.info("Catalogue initialise avec {} huiles", paints.count());
        }
        if (recipes.count() == 0) {
            recipes.saveAll(defaultRecipes());
            log.info("{} recettes d'exemple ajoutees", recipes.count());
        }
        syncCatalogue();
        backfillTints();
        seedMissingPalettes();
    }

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
     * ecouvillons, la teinte diluee, la possession, ses notes, et les tubes qu'il a
     * saisis lui-meme.</p>
     */
    private void syncCatalogue() {
        if (paints.count() == 0) {
            return;
        }

        Map<String, OilPaint> fromFiles = new HashMap<>();
        Set<String> coveredBrands = new HashSet<>();
        for (OilPaint reference : catalogLoader.load()) {
            fromFiles.put(key(reference), reference);
            coveredBrands.add(reference.getBrand());
        }

        List<OilPaint> refreshed = new ArrayList<>();
        List<OilPaint> obsolete = new ArrayList<>();

        for (OilPaint stored : paints.findAll()) {
            OilPaint reference = fromFiles.get(key(stored));
            if (reference != null) {
                if (refreshIdentity(stored, reference)) {
                    refreshed.add(stored);
                }
            } else if (!stored.isUserAdded() && coveredBrands.contains(stored.getBrand())) {
                obsolete.add(stored);
            }
        }

        List<OilPaint> added = fromFiles.values().stream()
                .filter(reference -> paints.findFirstByBrandIgnoreCaseAndNameIgnoreCase(
                        reference.getBrand(), reference.getName()).isEmpty())
                .toList();

        if (!refreshed.isEmpty()) {
            paints.saveAll(refreshed);
            log.info("{} fiches mises a jour d'apres les gammes", refreshed.size());
        }
        if (!added.isEmpty()) {
            paints.saveAll(added);
            log.info("{} huiles ajoutees au catalogue existant", added.size());
        }
        removeObsolete(obsolete);
    }

    /**
     * Reprend du fichier ce qui releve du fabricant, et rien d'autre.
     *
     * @return vrai si quelque chose a change
     */
    private static boolean refreshIdentity(OilPaint stored, OilPaint reference) {
        boolean changed = !stored.getCode().equals(reference.getCode())
                || !stored.getLegacyCode().equals(reference.getLegacyCode())
                || !stored.getPigments().equals(reference.getPigments())
                || stored.getOpacity() != reference.getOpacity()
                || stored.getDryingClass() != reference.getDryingClass();

        stored.setCode(reference.getCode());
        stored.setLegacyCode(reference.getLegacyCode());
        stored.setPigments(reference.getPigments());
        stored.setOpacity(reference.getOpacity());
        stored.setDryingClass(reference.getDryingClass());
        stored.setTintingStrength(reference.getTintingStrength());
        stored.setPigmentsVerified(reference.isPigmentsVerified());

        // La teinte relevee par le peintre l'emporte ; celle deduite se laisse corriger.
        if (stored.isColorDerived() && !reference.isColorDerived()) {
            stored.setHexColor(reference.getHexColor());
            stored.setColorDerived(false);
        }
        return changed;
    }

    /**
     * Retire les fiches perimees, en epargnant celles qui servent encore.
     *
     * <p>Un tube employe par une palette ou fige dans un projet ne peut pas disparaitre
     * sans emporter ce qui s'y refere. On demande donc d'abord lesquels sont utilises,
     * plutot que de tenter la suppression pour voir : une contrainte qui cede invalide
     * la session, et tout ce qui suit echoue avec elle.</p>
     */
    private void removeObsolete(List<OilPaint> obsolete) {
        if (obsolete.isEmpty()) {
            return;
        }
        Set<Long> inUse = paints.idsInUse();

        List<OilPaint> removable = obsolete.stream()
                .filter(paint -> !inUse.contains(paint.getId()))
                .toList();
        long kept = obsolete.size() - removable.size();

        if (!removable.isEmpty()) {
            paints.deleteAll(removable);
            log.info("{} fiches perimees retirees du catalogue", removable.size());
        }
        if (kept > 0) {
            log.info("{} fiches perimees conservees : elles servent encore a une palette ou un projet", kept);
        }
    }

    /**
     * Renseigne la teinte diluee des huiles qui n'en ont pas encore.
     *
     * <p>Ces teintes sont arrivees apres coup, avec le passage au modele de melange a
     * deux constantes. Les bases deja en service n'en ont donc aucune, et sans ce
     * rattrapage leurs proprietaires resteraient sur l'ancien modele indefiniment. Seule
     * la teinte est ecrite : tout ce que le peintre a pu corriger par ailleurs est
     * laisse en place.</p>
     */
    private void backfillTints() {
        Map<String, String> fromFiles = new HashMap<>();
        for (OilPaint reference : catalogLoader.load()) {
            if (reference.getTintHex() != null) {
                fromFiles.put(key(reference), reference.getTintHex());
            }
        }

        List<OilPaint> updated = paints.findAll().stream()
                .filter(paint -> paint.getTintHex() == null)
                .filter(paint -> fromFiles.containsKey(key(paint)))
                .peek(paint -> paint.setTintHex(fromFiles.get(key(paint))))
                .toList();

        if (!updated.isEmpty()) {
            paints.saveAll(updated);
            log.info("Teinte diluee renseignee sur {} huiles : melange a deux constantes actif", updated.size());
        }
    }

    private static String key(OilPaint paint) {
        return paint.getBrand() + "|" + paint.getName();
    }

    /**
     * Ajoute les palettes de reference absentes, une par une.
     *
     * <p>Volontairement pas un "tout ou rien" sur une base vide : une palette de
     * reference ajoutee dans une version ulterieure doit pouvoir arriver chez un
     * peintre qui utilise deja l'application, sans toucher a celles qu'il a
     * composees lui-meme. Une palette supprimee exprès reapparaitra au demarrage
     * suivant : renommez-la plutot que de la supprimer si elle vous gene.</p>
     */
    private void seedMissingPalettes() {
        for (Palette candidate : referencePalettes()) {
            if (!palettes.existsByNameIgnoreCase(candidate.getName())) {
                palettes.save(candidate);
                log.info("Palette de reference ajoutee : {}", candidate.getName());
            }
        }
    }

    /**
     * Deux palettes de demarrage, pour montrer l'usage : un ensemble se choisit pour un
     * sujet, pas couleur par couleur.
     */
    private List<Palette> referencePalettes() {
        Palette flesh = new Palette("Carnations 1/10", "Visages et mains de buste");
        flesh.setNotes("""
                Le blanc et le rouge portent les lumieres, la terre d'ombre et le bleu de \
                Prusse portent les ombres. Tout le reste se trouve en melangeant ces deux \
                extremites : une palette courte donne des carnations plus coherentes qu'une \
                palette fournie.""");
        addAll(flesh,
                paint("Abteilung 502", "Basic Flesh Tone"),
                paint("Abteilung 502", "Flesh Shadow"),
                paint("Abteilung 502", "Light Flesh Tone"),
                paint("Winsor & Newton", "Naples Yellow"),
                paint("Winsor & Newton", "Alizarin Crimson"),
                paint("Winsor & Newton", "Burnt Umber"),
                paint("Winsor & Newton", "Prussian Blue"),
                paint("Winsor & Newton", "Titanium White"));

        Palette armour = new Palette("Blindage vert olive", "Vehicule 1/35");
        armour.setNotes("Filtres, jus et salissures. Le blanc ne sert qu'a casser le vert, jamais a eclaircir.");
        addAll(armour,
                paint("Winsor & Newton", "Olive Green"),
                paint("Winsor & Newton", "Yellow Ochre"),
                paint("Winsor & Newton", "Ivory Black"),
                paint("Abteilung 502", "Cassel Earth/Shadow Brown"),
                paint("Abteilung 502", "Starship Filth"),
                paint("Abteilung 502", "Red Ochre/Light Rust"),
                paint("Gamblin", "Titanium White"));

        return List.of(zorn(), flesh, armour);
    }

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
     * cadmium ecarlate en est le substitut usuel. Attention, c'est lui qui impose
     * le rythme de la palette, les cadmiums etant les pigments les plus lents a secher.</p>
     */
    private Palette zorn() {
        Palette palette = new Palette("Palette Zorn (Winsor & Newton)", "Carnations et portraits");
        palette.setNotes("""
                Quatre tubes : blanc de titane, ocre jaune, cadmium ecarlate, noir d'ivoire.

                Le noir ne sert pas a noircir, il tient le role du bleu. Sorti du tube il est \
                chaud ; coupe de blanc il donne des gris franchement bleutes, et mele a \
                l'ocre des verts sourds. Les ombres d'une carnation se font donc au noir, \
                sans jamais ajouter de bleu.

                Cette divergence entre le tube et le melange est reelle, et c'est ce que le \
                modele a deux constantes de l'application sait reproduire.

                Zorn peignait au vermillon ; le cadmium ecarlate le remplace. C'est le \
                pigment le plus lent de la palette, il commande le planning de la seance.""");
        addAll(palette,
                paint("Winsor & Newton", "Titanium White"),
                paint("Winsor & Newton", "Yellow Ochre"),
                paint("Winsor & Newton", "Cadmium Scarlet"),
                paint("Winsor & Newton", "Ivory Black"));
        return palette;
    }

    private static void addAll(Palette palette, OilPaint... found) {
        for (OilPaint paint : found) {
            if (paint != null) {
                palette.add(paint);
            }
        }
    }

    /** Renvoie {@code null} plutot que d'echouer : une palette d'exemple incomplete
     *  vaut mieux qu'un demarrage bloque parce qu'un nom de tube a change. */
    private OilPaint paint(String brand, String name) {
        return paints.findFirstByBrandIgnoreCaseAndNameIgnoreCase(brand, name)
                .orElseGet(() -> {
                    log.warn("Palette d'exemple : '{} - {}' introuvable dans le catalogue", brand, name);
                    return null;
                });
    }

    private static List<Recipe> defaultRecipes() {
        Recipe flesh = new Recipe("Carnation 1/10 a l'huile", "Visage de buste");
        flesh.setNotes("""
                Base acrylique poncee et vernie brillante, seche depuis 24 h.
                Tout le modele repose sur des couches tres fines : on construit la profondeur
                par accumulation, jamais en une seule passe chargee.""");
        flesh.addStep(step(Technique.BASE_LAYER, "Basic Flesh Tone + une pointe de Naples Yellow",
                Medium.ODORLESS_THINNER, 0.25, LayerThickness.THIN,
                "Aplat tres fin, on doit encore deviner la base acrylique."));
        flesh.addStep(step(Technique.DOT_FADING, "Points de Deep Shadow Flesh, Naples Yellow et Alizarin Crimson",
                Medium.NONE, 0.0, LayerThickness.THIN,
                "Rouge sur les pommettes, le nez et les oreilles ; jaune sur le front."));
        flesh.addStep(step(Technique.BLENDING, "Fondu au pinceau propre",
                Medium.NONE, 0.0, LayerThickness.THIN,
                "Tirer les transitions tant que la couche est ouverte, essuyer le pinceau a chaque passage."));
        flesh.addStep(step(Technique.PIN_WASH, "Burnt Umber + une pointe de Prussian Blue",
                Medium.ODORLESS_THINNER, 0.90, LayerThickness.GLAZE,
                "Creux des paupieres, ailes du nez, commissures."));
        flesh.addStep(step(Technique.HIGHLIGHT, "Snow White + Basic Flesh Tone",
                Medium.NONE, 0.0, LayerThickness.THIN,
                "Arete du nez, arcade, levre inferieure. Trois points suffisent."));

        Recipe armour = new Recipe("Usure d'un blindage vert olive", "Vehicule 1/35");
        armour.setNotes("Enchainement classique : filtre, jus, coulures, puis poussieres.");
        armour.addStep(step(Technique.FILTER, "Olive Green tres dilue",
                Medium.ODORLESS_THINNER, 0.94, LayerThickness.GLAZE,
                "Unifie les modules de camouflage sans les effacer."));
        armour.addStep(step(Technique.PIN_WASH, "Cassel Earth/Shadow Brown",
                Medium.ODORLESS_THINNER, 0.90, LayerThickness.GLAZE,
                "Uniquement dans les lignes de structure et autour des rivets."));
        armour.addStep(step(Technique.STREAKING_GRIME, "Starship Filth",
                Medium.ODORLESS_THINNER, 0.75, LayerThickness.THIN,
                "Traits verticaux depuis les asperites, tires vers le bas apres deux minutes."));
        armour.addStep(step(Technique.GLAZE, "Rust dans les zones d'ecaillage",
                Medium.ODORLESS_THINNER, 0.85, LayerThickness.GLAZE,
                "Toujours au-dessus de l'ecaillage, jamais en dessous."));

        return List.of(flesh, armour);
    }

    private static RecipeStep step(Technique technique, String mix, Medium medium,
                                   double ratio, LayerThickness thickness, String notes) {
        RecipeStep step = new RecipeStep(technique, mix, medium, ratio, thickness);
        step.setNotes(notes);
        return step;
    }
}
