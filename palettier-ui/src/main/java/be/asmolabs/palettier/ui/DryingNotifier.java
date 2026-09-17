package be.asmolabs.palettier.ui;

import be.asmolabs.palettier.core.service.ReadinessWatch;
import be.asmolabs.palettier.core.service.ReadinessWatch.Freed;
import be.asmolabs.palettier.core.service.WorkbenchService;
import java.awt.EventQueue;
import java.awt.Image;
import java.awt.SystemTray;
import java.awt.TrayIcon;
import java.io.InputStream;
import java.time.Duration;
import java.util.List;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import javax.imageio.ImageIO;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Component;

/**
 * Previent quand une zone redevient reprenable, meme si l'on ne regarde pas.
 *
 * <p>C'est la difference entre savoir et devoir aller voir. Le sechage a l'huile se
 * compte en heures ou en jours : personne ne rafraichit un ecran pendant six heures pour
 * apprendre que la cape est prete. La surveillance tourne donc en fond, et ne dit que ce
 * qui vient de basculer -- {@link ReadinessWatch} se charge de ne rien repeter.</p>
 *
 * <p>L'avis passe par la zone de notification du systeme quand la plateforme la propose.
 * Quand elle ne la propose pas, il n'y a pas de repli : une fenetre surgissante pendant
 * qu'on peint serait pire que le silence. La section Aujourd'hui reste de toute facon la
 * reponse complete.</p>
 */
@Component
public class DryingNotifier {

    private static final Logger log = LoggerFactory.getLogger(DryingNotifier.class);

    /** Le sechage ne se joue pas a la minute : inutile de reveiller la base plus souvent. */
    private static final Duration INTERVAL = Duration.ofMinutes(5);

    private final WorkbenchService workbench;
    private final ReadinessWatch watch;

    private ScheduledExecutorService scheduler;
    private TrayIcon trayIcon;

    public DryingNotifier(WorkbenchService workbench, ReadinessWatch watch) {
        this.workbench = workbench;
        this.watch = watch;
    }

    @EventListener
    public void onStageReady(StageReadyEvent event) {
        scheduler = Executors.newSingleThreadScheduledExecutor(runnable -> {
            Thread thread = new Thread(runnable, "drying-watch");
            thread.setDaemon(true);
            return thread;
        });
        scheduler.scheduleWithFixedDelay(this::check,
                INTERVAL.toSeconds(), INTERVAL.toSeconds(), TimeUnit.SECONDS);
    }

    private void check() {
        try {
            List<Freed> freed = watch.newlyReady(workbench.bench());
            if (!freed.isEmpty()) {
                announce(freed);
            }
        } catch (RuntimeException e) {
            // Une surveillance de fond qui leve une exception se tait pour de bon :
            // l'executeur l'annule. On journalise et on continue.
            log.warn("Surveillance du sechage interrompue par une erreur, elle reprendra", e);
        }
    }

    private void announce(List<Freed> freed) {
        String body = freed.size() == 1
                ? freed.getFirst().label()
                : freed.stream().map(Freed::label).reduce((a, b) -> a + "\n" + b).orElse("");
        String title = freed.size() == 1
                ? "Une zone est reprenable"
                : "%d zones sont reprenables".formatted(freed.size());

        log.info("{} : {}", title, body.replace("\n", ", "));
        EventQueue.invokeLater(() -> show(title, body));
    }

    private void show(String title, String body) {
        try {
            if (!SystemTray.isSupported()) {
                return;
            }
            if (trayIcon == null) {
                // Cree au premier avis seulement : tant qu'il n'y a rien a dire, rien
                // n'a a s'installer dans la barre du systeme.
                Image image = icon();
                if (image == null) {
                    return;
                }
                trayIcon = new TrayIcon(image, "Palettier");
                trayIcon.setImageAutoSize(true);
                SystemTray.getSystemTray().add(trayIcon);
            }
            trayIcon.displayMessage(title, body, TrayIcon.MessageType.INFO);
        } catch (Exception e) {
            log.debug("Notification systeme indisponible", e);
        }
    }

    private static Image icon() {
        try (InputStream stream = DryingNotifier.class.getResourceAsStream("icon/palettier-64.png")) {
            return stream == null ? null : ImageIO.read(stream);
        } catch (Exception e) {
            log.debug("Icone de notification illisible", e);
            return null;
        }
    }
}
