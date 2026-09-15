package be.asmolabs.palettier.core.repository;

import be.asmolabs.palettier.core.domain.Recipe;
import java.util.List;
import org.springframework.data.jpa.repository.JpaRepository;

public interface RecipeRepository extends JpaRepository<Recipe, Long> {

    List<Recipe> findAllByOrderByNameAsc();
}
