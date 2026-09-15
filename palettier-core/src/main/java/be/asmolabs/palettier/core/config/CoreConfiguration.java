package be.asmolabs.palettier.core.config;

import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.repository.OilPaintRepository;
import be.asmolabs.palettier.core.service.DryingProperties;
import org.springframework.boot.persistence.autoconfigure.EntityScan;
import org.springframework.boot.context.properties.EnableConfigurationProperties;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;
import org.springframework.data.jpa.repository.config.EnableJpaRepositories;

/**
 * Point d'entree du module metier : a importer depuis l'application hote, qui n'a
 * ainsi pas besoin de connaitre l'organisation interne des paquets.
 */
@Configuration(proxyBeanMethods = false)
@ComponentScan("be.asmolabs.palettier.core")
@EntityScan(basePackageClasses = OilPaint.class)
@EnableJpaRepositories(basePackageClasses = OilPaintRepository.class)
@EnableConfigurationProperties(DryingProperties.class)
public class CoreConfiguration {
}
