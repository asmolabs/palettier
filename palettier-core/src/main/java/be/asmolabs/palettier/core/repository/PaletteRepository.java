package be.asmolabs.palettier.core.repository;

import be.asmolabs.palettier.core.domain.Palette;
import java.util.List;
import org.springframework.data.jpa.repository.JpaRepository;

public interface PaletteRepository extends JpaRepository<Palette, Long> {

    List<Palette> findAllByOrderByNameAsc();

    boolean existsByNameIgnoreCase(String name);
}
