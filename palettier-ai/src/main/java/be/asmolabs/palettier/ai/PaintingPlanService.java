package be.asmolabs.palettier.ai;

import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.image.ImagePalette;
import be.asmolabs.palettier.core.image.Photos;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.ColorMixService;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import java.util.ArrayList;
import java.util.List;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.ai.chat.client.ChatClient;
import org.springframework.ai.chat.prompt.ChatOptions;
import org.springframework.core.io.ByteArrayResource;
import org.springframework.util.MimeTypeUtils;
import org.springframework.beans.BeansException;
import org.springframework.beans.factory.ObjectProvider;
import org.springframework.stereotype.Service;

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
@Service
public class PaintingPlanService {

    private static final Logger log = LoggerFactory.getLogger(PaintingPlanService.class);

    /**
     * Plafond de generation. Un plan a huit zones fait trois mille mots de JSON : avec la
     * limite par defaut de certains moteurs, la reponse est coupee en plein objet et rien
     * n'est exploitable. On demande donc explicitement de la place.
     */
    private static final int MAX_TOKENS = 16384;

    /** Nombre de teintes relevees sur la photo et soumises au modele. */
    private static final int MEASURED_COLOURS = 10;

    private static final String SYSTEM_PROMPT = """
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
            demande pas : un aplat de bois sec n'a pas de variation locale. Une seule ombre et une seule lumiere suffisent a
            poser un volume, jamais a le rendre : c'est l'etagement des valeurs qui separe
            une piece plate d'une piece modelee. La derniere ombre se loge dans les creux
            les plus fermes, le dernier point lumineux sur une arete seulement.

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
            """;

    private final ObjectProvider<ChatClient.Builder> chatClientBuilder;
    private final OilPainterTools tools;
    private final ColorMixService mixer;
    private final AiSettings settings;

    public PaintingPlanService(ObjectProvider<ChatClient.Builder> chatClientBuilder,
                               OilPainterTools tools,
                               ColorMixService mixer,
                               AiSettings settings) {
        this.chatClientBuilder = chatClientBuilder;
        this.tools = tools;
        this.mixer = mixer;
        this.settings = settings;
    }

    /**
     * Vrai si un moteur de conversation est configure. Sans cela le reste de
     * l'application fonctionne a l'identique : l'assistant est un supplement, pas une
     * dependance.
     */
    public boolean isAvailable() {
        return builderOrNull() != null;
    }

    /**
     * Le bean constructeur du client existe des que Spring AI est au classpath, mais il
     * reste inconstructible tant qu'aucun moteur n'est selectionne : sa fabrication echoue
     * alors sur l'absence de {@code ChatModel}. Une simple recherche de bean ne suffit donc
     * pas a decider de la disponibilite, il faut tenter la construction.
     */
    private ChatClient.Builder builderOrNull() {
        try {
            return chatClientBuilder.getIfAvailable();
        } catch (BeansException e) {
            log.debug("Aucun moteur de conversation utilisable", e);
            return null;
        }
    }

    /**
     * @param subject   description libre du sujet, par exemple "buste de grognard
     *                  napoleonien, manteau bleu et bonnet a poil"
     * @param palette   palette avec laquelle le plan doit etre realisable
     * @param maxPaints nombre maximal de tubes par melange
     */
    public PaintingPlan plan(String subject, Palette palette, int maxPaints) {
        return plan(subject, palette, maxPaints, null, List.of());
    }

    /**
     * @param figurine   photo de la piece a peindre, ou {@code null}
     * @param references photos de ce que l'on cherche a obtenir, eventuellement vide
     */
    public PaintingPlan plan(String subject, Palette palette, int maxPaints,
                             PhotoInput figurine, List<PhotoInput> references) {
        return plan(subject, palette, maxPaints, figurine, references, null);
    }

    /**
     * @param model nom du modele a employer pour cette demande, ou {@code null} pour
     *              celui de la configuration. La qualite du plan en depend fortement :
     *              un petit modele decoupe grossierement.
     */
    public PaintingPlan plan(String subject, Palette palette, int maxPaints,
                             PhotoInput figurine, List<PhotoInput> references, String model) {
        ChatClient.Builder builder = builderOrNull();
        if (builder == null) {
            throw new IllegalStateException(
                    "Aucun moteur de conversation configure. Renseignez spring.ai.model.chat.");
        }

        ChatClient client = builder
                .defaultSystem(SYSTEM_PROMPT)
                .defaultTools(tools)
                .build();

        // Reduction imposee ici, et non laissee a l'appelant : c'est une contrainte du
        // transport, pas un choix d'interface.
        List<PhotoInput> photos = new ArrayList<>();
        if (figurine != null) {
            photos.add(reduced(figurine));
        }
        if (references != null) {
            references.forEach(reference -> photos.add(reduced(reference)));
        }

        String question = userPrompt(subject, palette, figurine != null, photos,
                measuredColours(figurine, references));
        // Priorite : le modele demande pour cette requete, sinon celui choisi dans les
        // parametres pour la session, sinon celui du fichier de configuration.
        String chosen = model != null && !model.isBlank() ? model : settings.model().orElse(null);

        ChatOptions.Builder<?> options = ChatOptions.builder().maxTokens(MAX_TOKENS);
        if (chosen != null) {
            options.model(chosen);
        }

        log.info("Plan de peinture demande pour {} avec la palette {}, {} photo(s), modele {}",
                subject, palette.getName(), photos.size(), chosen == null ? "(par defaut)" : chosen);

        PlanDraft draft;
        try {
            draft = client.prompt()
                    .options(options)
                    .user(spec -> {
                        spec.text(question);
                        for (PhotoInput photo : photos) {
                            spec.media(MimeTypeUtils.parseMimeType(PhotoInput.MIME_TYPE),
                                    new ByteArrayResource(photo.data()));
                        }
                    })
                    .call()
                    // Le schema est remis au moteur comme contrainte de generation, et non
                    // ajoute au prompt comme consigne. La difference est de nature : une
                    // consigne se suit a peu pres, une contrainte interdit au moteur de
                    // produire le jeton qui casserait le JSON. Chez Ollama, cela passe par
                    // le champ "format" ; les moteurs qui ne le gerent pas retombent sur la
                    // consigne en clair, sans rien changer d'autre.
                    //
                    // La validation qui suit rattrape ce que la contrainte ne couvre pas :
                    // un modele a mode de reflexion peut repondre a cote en texte libre,
                    // auquel cas l'erreur lui est renvoyee et la demande rejouee.
                    .entity(PlanDraft.class, spec -> spec.useProviderStructuredOutput().validateSchema());
        } catch (RuntimeException e) {
            log.warn("Plan inexploitable renvoye par le modele {}", chosen == null ? "(par defaut)" : chosen, e);
            throw new IllegalStateException(
                    "Le modele n'a pas produit de plan exploitable, meme apres plusieurs tentatives. "
                    + "Essayez un modele plus capable, ou moins de zones en precisant le sujet. ("
                    + e.getClass().getSimpleName() + ")", e);
        }

        return enrich(subject, palette, draft, maxPaints);
    }

    /**
     * La palette est enumeree dans la question elle-meme plutot que laissee a un appel
     * d'outil : c'est l'information dont le modele a besoin a coup sur, autant la lui
     * donner d'emblee que dependre de son initiative.
     */
    /**
     * Les teintes reellement presentes sur les photos fournies.
     *
     * <p>C'est la difference entre un plan fonde sur la piece et un plan fonde sur l'idee
     * qu'on se fait du sujet. La reference l'emporte sur l'etat actuel quand les deux sont
     * la : on peint vers ce qu'on veut obtenir.</p>
     */
    private record Measured(List<ImagePalette.DominantColour> colours, boolean fromReference) {

        static final Measured NONE = new Measured(List.of(), false);

        boolean isEmpty() {
            return colours.isEmpty();
        }
    }

    private static Measured measuredColours(PhotoInput figurine, List<PhotoInput> references) {
        boolean hasReference = references != null && !references.isEmpty();
        byte[] source = hasReference
                ? references.getFirst().data()
                : figurine == null ? null : figurine.data();
        if (source == null) {
            return Measured.NONE;
        }
        try {
            return new Measured(ImagePalette.dominant(source, MEASURED_COLOURS), hasReference);
        } catch (RuntimeException e) {
            log.warn("Teintes de la photo illisibles, le plan se fera sans", e);
            return Measured.NONE;
        }
    }

    private static String userPrompt(String subject, Palette palette, boolean hasFigurine,
                                     List<PhotoInput> photos,
                                     Measured measured) {
        StringBuilder prompt = new StringBuilder();

        if (!measured.isEmpty()) {
            // La distinction est decisive : les teintes d'une reference sont des cibles,
            // celles d'une piece nue ne sont que du metal ou de l'appret.
            prompt.append(measured.fromReference()
                    ? "Teintes relevees par l'application sur la REFERENCE, avec la part de "
                      + "l'image qu'elles occupent. Ce sont des mesures, et ce sont tes cibles :\n"
                    : "Teintes relevees par l'application sur la PIECE TELLE QU'ELLE EST, avec la "
                      + "part de l'image qu'elles occupent :\n");
            for (ImagePalette.DominantColour colour : measured.colours()) {
                prompt.append("  ").append(colour.color().toHex())
                        .append("  ").append(Math.round(colour.share() * 100)).append(" %\n");
            }
            prompt.append(measured.fromReference()
                    ? "Appuie tes couleurs sur ces mesures plutot que sur l'idee generale que tu "
                      + "te fais du sujet.\n"
                    : "Attention : c'est l'etat de depart, pas la cible. Si la piece est nue -- "
                      + "metal, resine, appret gris -- ces teintes ne disent rien de ce qu'il faut "
                      + "peindre, et tu dois les ignorer completement pour proposer les couleurs "
                      + "du sujet represente. Elles ne servent que si la piece est deja peinte.\n");
            prompt.append("Une part tres importante correspond souvent au fond, ignore-la.\n\n");
        }

        // Les images arrivent dans l'ordre ou elles sont ajoutees, sans etiquette : sans
        // cette enumeration, le modele ne sait pas laquelle est la piece et laquelle est
        // la reference.
        if (!photos.isEmpty()) {
            prompt.append("Images jointes, dans l'ordre :\n");
            for (int i = 0; i < photos.size(); i++) {
                prompt.append("  Image ").append(i + 1).append(" : ")
                        .append(photos.get(i).caption()).append('\n');
            }
            prompt.append('\n');
            if (hasFigurine && photos.size() > 1) {
                prompt.append("Compare l'etat actuel de la piece aux references, et batis le "
                        + "plan pour combler l'ecart.\n\n");
            } else if (hasFigurine) {
                prompt.append("Decoupe la piece en zones d'apres ce que tu vois.\n\n");
            }
        }

        prompt.append("Sujet a peindre : ")
                .append(subject == null || subject.isBlank() ? "voir les images jointes" : subject)
                .append("\n\n");
        prompt.append("Palette disponible, ").append(palette.getName()).append(" :\n");
        for (OilPaint paint : palette.getPaints()) {
            prompt.append("- ").append(paint.displayName())
                    .append(" : ").append(paint.getHexColor())
                    .append(", pigments ").append(String.join(" ", paint.getPigments()))
                    .append(", sechage ").append(paint.getDryingClass().label().toLowerCase())
                    .append('\n');
        }
        prompt.append("\nDecompose le sujet en zones et donne pour chacune la base, l'ombre et la lumiere.");
        return prompt.toString();
    }

    // --- Deuxieme temps : les melanges, calcules et non demandes -------------

    /** Package-prive : c'est la partie hors ligne du service, et elle merite d'etre testee sans moteur. */
    PaintingPlan enrich(String subject, Palette palette, PlanDraft draft, int maxPaints) {
        List<PaintingPlan.Zone> zones = new ArrayList<>();
        for (PlanDraft.ZoneDraft zone : draft.zones()) {
            PaintingPlan.Layer base = layer("Base", zone.base(), palette, maxPaints);
            zones.add(new PaintingPlan.Zone(
                    zone.name(), zone.material(), zone.note(), base,
                    List.of(step("Ombre 1", zone.shadow1(), base, zone.shadow2(), palette, maxPaints),
                            step("Ombre 2", zone.shadow2(), base, zone.shadow1(), palette, maxPaints)),
                    List.of(step("Lumiere 1", zone.highlight1(), base, zone.highlight2(), palette, maxPaints),
                            step("Lumiere 2", zone.highlight2(), base, zone.highlight1(), palette, maxPaints)),
                    accents(zone, palette, maxPaints)));
        }
        return new PaintingPlan(subject, palette.getName(), draft.approach(), List.copyOf(zones));
    }

    /**
     * Une couche du degrade, avec un repli si le modele a saute un champ.
     *
     * <p>Plutot que de renoncer, on interpole : la couche manquante devient le melange a
     * parts egales de la base et de l'autre couche du meme cote. Ce n'est pas un jugement
     * invente, c'est le milieu de deux couleurs que le modele a lui-meme choisies, calcule
     * comme n'importe quel autre melange.</p>
     */
    /**
     * Les variations locales effectivement proposees.
     *
     * <p>Contrairement aux couches de l'echelle, on ne comble pas les manquantes : une
     * variation locale est une observation, elle ne s'interpole pas. Une zone qui n'en
     * demande pas n'en recoit aucune.</p>
     */
    private List<PaintingPlan.Layer> accents(PlanDraft.ZoneDraft zone, Palette palette, int maxPaints) {
        List<PaintingPlan.Layer> accents = new ArrayList<>();
        for (PlanDraft.AccentDraft accent : List.of(
                java.util.Optional.ofNullable(zone.accent1()),
                java.util.Optional.ofNullable(zone.accent2()),
                java.util.Optional.ofNullable(zone.accent3()))
                .stream().flatMap(java.util.Optional::stream).toList()) {
            if (accent.hex() == null || accent.hex().isBlank()) {
                continue;
            }
            String name = accent.name() == null || accent.name().isBlank()
                    ? "Variation " + (accents.size() + 1) : accent.name();
            accents.add(layer(name,
                    new PlanDraft.LayerDraft(accent.hex(), accent.technique(), accent.note()),
                    palette, maxPaints));
        }
        return List.copyOf(accents);
    }

    private PaintingPlan.Layer step(String role, PlanDraft.LayerDraft draft, PaintingPlan.Layer base,
                                    PlanDraft.LayerDraft sibling, Palette palette, int maxPaints) {
        if (draft != null && draft.hex() != null && !draft.hex().isBlank()) {
            return layer(role, draft, palette, maxPaints);
        }
        if (sibling == null || sibling.hex() == null || sibling.hex().isBlank()) {
            return layer(role, new PlanDraft.LayerDraft(base.target().toHex(), "", "Couche absente de la reponse."),
                    palette, maxPaints);
        }
        Rgb between = Colors.mix(List.of(base.target(), parse(sibling.hex())), List.of(1.0, 1.0));
        return layer(role, new PlanDraft.LayerDraft(between.toHex(), sibling.technique(),
                "Ton interpole entre la base et l'autre couche : absent de la reponse du modele."),
                palette, maxPaints);
    }

    private PaintingPlan.Layer layer(String role, PlanDraft.LayerDraft draft,
                                     Palette palette, int maxPaints) {
        Rgb target = parse(draft == null ? null : draft.hex());
        String technique = draft == null ? "" : draft.technique();
        String note = draft == null ? "" : draft.note();

        List<MixSuggestion> found = mixer.suggestMixes(target, palette.getPaints(), 1, maxPaints);
        if (found.isEmpty()) {
            return new PaintingPlan.Layer(role, target, technique, note, null, target, 0);
        }
        MixSuggestion best = found.getFirst();
        return new PaintingPlan.Layer(role, target, technique, note,
                best, best.color(), Colors.deltaE2000(target, best.color()));
    }

    private static PhotoInput reduced(PhotoInput photo) {
        return new PhotoInput(Photos.prepare(photo.data()), photo.caption());
    }

    /** Un modele de langage se trompe parfois de format : on n'en fait pas un incident. */
    private static Rgb parse(String hex) {
        try {
            return Rgb.ofHex(hex);
        } catch (RuntimeException e) {
            log.warn("Couleur inexploitable renvoyee par le modele : {}", hex);
            return new Rgb(0.5, 0.5, 0.5);
        }
    }
}
