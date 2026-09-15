package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.repository.OilPaintRepository;
import be.asmolabs.palettier.core.service.MixModels.PaintMatch;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

/** Acces au catalogue d'huiles et recherche par proximite de couleur. */
@Service
@Transactional(readOnly = true)
public class PaintCatalogService {

    private final OilPaintRepository repository;

    public PaintCatalogService(OilPaintRepository repository) {
        this.repository = repository;
    }

    public List<OilPaint> findAll() {
        return repository.findAllByOrderByBrandAscNameAsc();
    }

    public List<OilPaint> findInStock() {
        return repository.findByInStockTrueOrderByBrandAscNameAsc();
    }

    public List<OilPaint> search(String term) {
        return term == null || term.isBlank() ? findAll() : repository.search(term.trim());
    }

    public Optional<OilPaint> findById(long id) {
        return repository.findById(id);
    }

    @Transactional
    public OilPaint save(OilPaint paint) {
        return repository.save(paint);
    }

    /**
     * Enregistre la couleur relevee sur un ecouvillon comme ton de masse d'un tube.
     *
     * <p>Remplace la valeur du catalogue, qui n'est au mieux qu'une approximation d'un
     * nuancier imprime, par une mesure faite sur la peinture du peintre, sous sa lumiere.</p>
     */
    @Transactional
    public OilPaint recordMasstone(OilPaint paint, Rgb measured) {
        paint.setHexColor(measured.toHex());
        paint.setColorDerived(false);
        return repository.save(paint);
    }

    /**
     * Enregistre la couleur relevee comme teinte diluee d'un tube.
     *
     * <p>C'est l'information qui fait passer ce tube au modele de melange a deux
     * constantes. L'ecouvillon doit etre prepare dans la proportion de reference, une
     * part de couleur pour neuf de blanc de titane : c'est cette concentration que le
     * calcul suppose.</p>
     */
    @Transactional
    public OilPaint recordTint(OilPaint paint, Rgb measured) {
        paint.setTintHex(measured.toHex());
        return repository.save(paint);
    }

    /**
     * Declare la possession d'un tube.
     *
     * <p>Le catalogue livre tout marque comme possede, ce qui ne veut rien dire : c'est
     * au peintre de dire ce qu'il a vraiment. Tant qu'il ne l'a pas fait, le filtre
     * "mes tubes" est sans effet.</p>
     */
    @Transactional
    public OilPaint setOwned(OilPaint paint, boolean owned) {
        paint.setInStock(owned);
        return repository.save(paint);
    }

    @Transactional
    public int setOwned(List<OilPaint> paints, boolean owned) {
        paints.forEach(paint -> paint.setInStock(owned));
        repository.saveAll(paints);
        return paints.size();
    }

    /** Repart de zero : plus aucun tube declare. Le point de depart d'un inventaire. */
    @Transactional
    public int declareNothingOwned() {
        List<OilPaint> owned = repository.findByInStockTrueOrderByBrandAscNameAsc();
        return setOwned(owned, false);
    }

    public long countOwned() {
        return repository.findByInStockTrueOrderByBrandAscNameAsc().size();
    }

    @Transactional
    public void delete(OilPaint paint) {
        repository.delete(paint);
    }

    /**
     * Classe les huiles par ecart percu a une couleur cible : repond a la question
     * "quel tube de mon etagere se rapproche le plus de cette teinte ?".
     */
    public List<PaintMatch> findClosest(Rgb target, boolean onlyInStock, int limit) {
        List<OilPaint> candidates = onlyInStock ? findInStock() : findAll();
        return candidates.stream()
                .map(paint -> new PaintMatch(paint, Colors.deltaE2000(target, paint.color())))
                .sorted(Comparator.comparingDouble(PaintMatch::deltaE))
                .limit(limit)
                .toList();
    }
}
