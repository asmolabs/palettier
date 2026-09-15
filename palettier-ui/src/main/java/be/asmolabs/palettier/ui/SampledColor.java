package be.asmolabs.palettier.ui;

import be.asmolabs.palettier.core.color.Rgb;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import org.springframework.stereotype.Component;

/**
 * La derniere teinte relevee a la pipette, partagee entre les ecrans.
 *
 * <p>Mesurer et enregistrer sont deux gestes distincts : on releve une couleur sur une
 * photo dans la Pipette, on l'attribue a un tube dans le Catalogue, la ou l'on voit
 * justement ce qui manque. Cet objet est le seul lien entre les deux, et evite que
 * chacun des deux ecrans ait a connaitre l'autre.</p>
 */
@Component
public class SampledColor {

    private final ObjectProperty<Rgb> value = new SimpleObjectProperty<>();

    public ObjectProperty<Rgb> valueProperty() {
        return value;
    }

    public Rgb get() {
        return value.get();
    }

    public void set(Rgb color) {
        value.set(color);
    }
}
