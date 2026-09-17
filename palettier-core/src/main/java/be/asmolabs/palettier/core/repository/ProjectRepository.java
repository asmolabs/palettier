package be.asmolabs.palettier.core.repository;

import be.asmolabs.palettier.core.domain.Project;
import java.util.List;
import java.util.Optional;
import org.springframework.data.jpa.repository.EntityGraph;
import org.springframework.data.jpa.repository.JpaRepository;

public interface ProjectRepository extends JpaRepository<Project, Long> {

    List<Project> findAllByOrderByCreatedAtDesc();

    /**
     * Un projet avec ses photos.
     *
     * <p>Elles sont paresseuses par defaut : c'est ce qui evite de lire toutes les images
     * du poste pour afficher une liste. L'ecran qui les montre les demande donc ici, en
     * une requete plutot qu'en autant de lectures qu'il y a d'images.</p>
     */
    @EntityGraph(attributePaths = "photos")
    Optional<Project> findWithPhotosById(Long id);

    boolean existsByNameIgnoreCase(String name);
}
