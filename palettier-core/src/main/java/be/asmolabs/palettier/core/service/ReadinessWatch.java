package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.service.WorkbenchService.Bench;
import be.asmolabs.palettier.core.service.WorkbenchService.PieceState;
import be.asmolabs.palettier.core.service.WorkbenchService.ZoneState;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import org.springframework.stereotype.Component;

/**
 * Ce qui vient de se liberer depuis le dernier regard.
 *
 * <p>Une zone recouvrable ne l'annonce qu'une fois. Sans cette memoire, une surveillance
 * qui repasse toutes les cinq minutes repeterait indefiniment que la cape est prete, et
 * l'on cesserait de la lire -- c'est le defaut qui rend une alerte inutile.</p>
 *
 * <p>Le premier passage ne signale rien. En ouvrant l'application, le peintre n'attend
 * pas la liste de tout ce qui a seche pendant son absence : il la voit a l'ecran. Une
 * alerte n'a de sens que pour ce qui bascule sous ses yeux.</p>
 */
@Component
public class ReadinessWatch {

    /** Une zone qui vient de se liberer, telle qu'on l'annonce. */
    public record Freed(String piece, String zone) {

        public String label() {
            return piece + " - " + zone;
        }
    }

    private Set<String> ready = Set.of();
    private boolean primed;

    /**
     * Les zones devenues disponibles depuis l'appel precedent.
     *
     * @return la liste, vide au premier appel comme lorsque rien n'a bouge
     */
    public synchronized List<Freed> newlyReady(Bench bench) {
        Set<String> now = new LinkedHashSet<>();
        List<Freed> freed = new ArrayList<>();

        for (PieceState piece : bench.pieces()) {
            for (ZoneState zone : piece.readyZones()) {
                String key = piece.project().getId() + "/" + zone.name();
                now.add(key);
                if (primed && !ready.contains(key)) {
                    freed.add(new Freed(piece.project().getName(), zone.name()));
                }
            }
        }

        ready = now;
        boolean first = !primed;
        primed = true;
        return first ? List.of() : List.copyOf(freed);
    }

    /** Oublie ce qui a ete vu. Le prochain regard repart comme au premier jour. */
    public synchronized void reset() {
        ready = Set.of();
        primed = false;
    }
}
