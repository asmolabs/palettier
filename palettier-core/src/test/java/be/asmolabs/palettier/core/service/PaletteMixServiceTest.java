package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.PaletteMix;
import be.asmolabs.palettier.core.domain.Ventilation;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import be.asmolabs.palettier.core.service.PaletteMixService.MixState;
import java.time.Duration;
import java.time.Instant;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.transaction.annotation.Transactional;

@SpringBootTest
@Transactional
class PaletteMixServiceTest {

    private static final Instant NOW = Instant.parse("2026-03-01T10:00:00Z");

    @Autowired
    private PaletteMixService mixes;

    private PaletteMix gris(DryingClass dryingClass, Workshop workshop) {
        return mixes.record("Gris rompu des ombres", Rgb.ofHex("#6B6259"),
                "2 parts de terre d'ombre + 1 part de blanc", dryingClass, workshop, NOW);
    }

    @Test
    @DisplayName("un melange frais est ouvert, et le restera quelques heures")
    void afreshMixIsOpen() {
        PaletteMix mix = gris(DryingClass.MEDIUM, Workshop.standard());

        MixState state = mixes.state(mix, NOW.plus(Duration.ofMinutes(30)));

        assertThat(state.isOpen()).isTrue();
        assertThat(state.isSpent()).isFalse();
        assertThat(state.workable()).isPositive();
    }

    @Test
    @DisplayName("passe le temps ouvert il ne se travaille plus, et plus tard il a pris")
    void amixCloses() {
        PaletteMix mix = gris(DryingClass.FAST, Workshop.standard());

        // Tres au-dela de tout jalon : il a pris, il n'a plus rien a faire la.
        MixState spent = mixes.state(mix, NOW.plus(Duration.ofDays(20)));
        assertThat(spent.isOpen()).isFalse();
        assertThat(spent.isSpent()).isTrue();
    }

    @Test
    @DisplayName("le temps ouvert depend de l'atelier du moment ou l'on a melange")
    void theworkshopOfTheMomentDecides() {
        PaletteMix cold = mixes.record("Froid", Rgb.ofHex("#6B6259"), "", DryingClass.MEDIUM,
                new Workshop(10, 80, Ventilation.CONFINED), NOW);
        PaletteMix warm = mixes.record("Chaud", Rgb.ofHex("#6B6259"), "", DryingClass.MEDIUM,
                new Workshop(28, 30, Ventilation.GOOD), NOW);

        Instant later = NOW.plus(Duration.ofHours(1));
        assertThat(mixes.state(cold, later).workable())
                .as("un atelier froid et humide garde le melange ouvert plus longtemps")
                .isGreaterThan(mixes.state(warm, later).workable());
    }

    @Test
    @DisplayName("une pate sur la palette tient plus longtemps que la meme couche sur la piece")
    void apuddleOutlivesAfilm() {
        PaletteMix mix = gris(DryingClass.MEDIUM, Workshop.standard());
        assertThat(mixes.state(mix, NOW).workable())
                .as("le tas n'est pas etale : il prend bien plus lentement")
                .isGreaterThan(Duration.ofHours(1));
    }

    @Test
    @DisplayName("faire le menage ne retire que ce qui a pris")
    void cleaningRemovesOnlyWhatIsSpent() {
        gris(DryingClass.MEDIUM, Workshop.standard());
        int before = mixes.findAll().size();

        // A l'instant meme, rien n'a pris : le menage ne doit rien emporter.
        assertThat(mixes.forgetSpent(NOW)).isZero();
        assertThat(mixes.findAll()).hasSize(before);

        // Trois semaines plus tard, le melange n'a plus rien a faire sur la palette.
        assertThat(mixes.forgetSpent(NOW.plus(Duration.ofDays(21)))).isEqualTo(before);
        assertThat(mixes.findAll()).isEmpty();
    }
}
