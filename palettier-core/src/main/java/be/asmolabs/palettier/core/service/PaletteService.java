package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.repository.PaletteRepository;
import java.util.List;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

/** Gestion des palettes du peintre. */
@Service
@Transactional(readOnly = true)
public class PaletteService {

    private final PaletteRepository repository;

    public PaletteService(PaletteRepository repository) {
        this.repository = repository;
    }

    public List<Palette> findAll() {
        return repository.findAllByOrderByNameAsc();
    }

    @Transactional
    public Palette create(String name, String purpose) {
        String unique = uniqueName(name);
        return repository.save(new Palette(unique, purpose));
    }

    @Transactional
    public Palette save(Palette palette) {
        return repository.save(palette);
    }

    @Transactional
    public void delete(Palette palette) {
        repository.delete(palette);
    }

    @Transactional
    public Palette addPaint(Palette palette, OilPaint paint) {
        palette.add(paint);
        return repository.save(palette);
    }

    @Transactional
    public Palette removePaint(Palette palette, OilPaint paint) {
        palette.remove(paint);
        return repository.save(palette);
    }

    /** Deux palettes du meme nom seraient impossibles a distinguer dans la liste. */
    private String uniqueName(String wanted) {
        String candidate = wanted.isBlank() ? "Nouvelle palette" : wanted.trim();
        String base = candidate;
        int suffix = 2;
        while (repository.existsByNameIgnoreCase(candidate)) {
            candidate = base + " " + suffix++;
        }
        return candidate;
    }
}
