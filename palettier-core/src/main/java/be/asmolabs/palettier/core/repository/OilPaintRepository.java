package be.asmolabs.palettier.core.repository;

import be.asmolabs.palettier.core.domain.OilPaint;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

public interface OilPaintRepository extends JpaRepository<OilPaint, Long> {

    List<OilPaint> findAllByOrderByBrandAscNameAsc();

    List<OilPaint> findByInStockTrueOrderByBrandAscNameAsc();

    Optional<OilPaint> findByBrandIgnoreCaseAndCodeIgnoreCase(String brand, String code);

    Optional<OilPaint> findFirstByBrandIgnoreCaseAndNameIgnoreCase(String brand, String name);

    /**
     * Identifiants des tubes auxquels une palette ou un projet se refere.
     *
     * <p>A consulter avant tout menage : tenter la suppression pour voir si elle passe
     * invalide la session des qu'une contrainte cede, et tout ce qui suit echoue.</p>
     */
    @Query("""
            select distinct paint.id from Palette palette join palette.paints paint
            union
            select distinct paint.id from Project project join project.paints paint
            """)
    Set<Long> idsInUse();

    @Query("""
            select p from OilPaint p
            where lower(p.name) like lower(concat('%', :term, '%'))
               or lower(p.brand) like lower(concat('%', :term, '%'))
               or lower(p.code) like lower(concat('%', :term, '%'))
               or lower(p.legacyCode) like lower(concat('%', :term, '%'))
            order by p.brand asc, p.name asc
            """)
    List<OilPaint> search(@Param("term") String term);
}
