package be.asmolabs.palettier.ui.component;

import be.asmolabs.palettier.core.color.Rgb;
import java.time.Duration;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.Locale;
import javafx.scene.paint.Color;

/** Mise en forme des valeurs affichees. */
public final class Formats {

    private static final DateTimeFormatter CLOCK =
            DateTimeFormatter.ofPattern("EEEE d MMMM 'a' HH'h'mm", Locale.FRENCH);

    private Formats() {
    }

    /** Duree en langage courant : "45 min", "6 h 30", "2 j 4 h", "3 semaines". */
    public static String duration(Duration duration) {
        long minutes = duration.toMinutes();
        if (minutes < 60) {
            return minutes + " min";
        }
        long hours = duration.toHours();
        if (hours < 48) {
            long remainder = minutes % 60;
            return remainder == 0 ? hours + " h" : "%d h %02d".formatted(hours, remainder);
        }
        long days = duration.toDays();
        if (days < 14) {
            long remainingHours = hours % 24;
            return remainingHours == 0 ? days + " jours" : "%d j %d h".formatted(days, remainingHours);
        }
        return "%d semaines".formatted(Math.round(days / 7.0));
    }

    /** Date et heure atteintes en partant de maintenant. */
    public static String clockAfter(Duration duration) {
        return CLOCK.format(LocalDateTime.now().plus(duration));
    }

    public static String percent(double ratio) {
        return "%.0f %%".formatted(ratio * 100);
    }

    public static Color toFx(Rgb rgb) {
        return Color.color(rgb.r(), rgb.g(), rgb.b());
    }

    public static Rgb fromFx(Color color) {
        return new Rgb(color.getRed(), color.getGreen(), color.getBlue());
    }
}
