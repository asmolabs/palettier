package be.asmolabs.palettier.core;

import be.asmolabs.palettier.core.config.CoreConfiguration;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.Import;

/** Application minimale servant de point d'ancrage aux tests d'integration du module. */
@SpringBootApplication
@Import(CoreConfiguration.class)
public class CoreTestApplication {
}
