package be.asmolabs.palettier.core.repository;

import be.asmolabs.palettier.core.domain.PaletteMix;
import java.util.List;
import org.springframework.data.jpa.repository.JpaRepository;

public interface PaletteMixRepository extends JpaRepository<PaletteMix, Long> {

    List<PaletteMix> findAllByOrderByMixedAtDesc();
}
