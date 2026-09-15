package be.asmolabs.palettier.ai;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatIllegalStateException;

import be.asmolabs.palettier.core.service.ColorMixService;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.ObjectProvider;
import org.springframework.beans.factory.support.StaticListableBeanFactory;

/**
 * Le comportement testable sans moteur : l'assistant doit etre inerte et le dire, pas
 * echouer sourdement ni empecher le reste de fonctionner.
 */
class PaintingPlanServiceTest {

    private final ObjectProvider<org.springframework.ai.chat.client.ChatClient.Builder> none =
            new StaticListableBeanFactory()
                    .getBeanProvider(org.springframework.ai.chat.client.ChatClient.Builder.class);

    private final PaintingPlanService service =
            new PaintingPlanService(none, null, new ColorMixService(), new AiSettings());

    @Test
    @DisplayName("sans moteur configure, l'assistant se declare indisponible")
    void unavailableWithoutAnEngine() {
        assertThat(service.isAvailable()).isFalse();
    }

    @Test
    @DisplayName("une demande sans moteur echoue avec un message exploitable, pas une erreur technique")
    void askingWithoutAnEngineExplainsWhy() {
        assertThatIllegalStateException()
                .isThrownBy(() -> service.plan("un buste", null, 3))
                .withMessageContaining("spring.ai.model.chat");
    }
}
