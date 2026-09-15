package be.asmolabs.palettier.core.repository;

import be.asmolabs.palettier.core.domain.Project;
import java.util.List;
import org.springframework.data.jpa.repository.JpaRepository;

public interface ProjectRepository extends JpaRepository<Project, Long> {

    List<Project> findAllByOrderByCreatedAtDesc();

    boolean existsByNameIgnoreCase(String name);
}
