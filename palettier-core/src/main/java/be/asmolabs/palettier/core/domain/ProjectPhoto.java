package be.asmolabs.palettier.core.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.Lob;
import jakarta.persistence.Table;
import java.time.Instant;

/**
 * Une photo attachee a un projet.
 *
 * <p>Les images sont reduites avant d'etre rangees : une base de donnees n'a pas a
 * porter des photos de telephone pleine resolution, et l'usage qu'on en fait -- se
 * rappeler d'ou l'on part et ou l'on en est -- ne le demande pas.</p>
 */
@Entity
@Table(name = "project_photo")
public class ProjectPhoto {

    /** Ce que la photo montre : la piece, ou la reference visee. */
    public enum Role {
        PIECE("La piece"),
        REFERENCE("Reference"),
        PROGRESS("Avancement");

        private final String label;

        Role(String label) {
            this.label = label;
        }

        public String label() {
            return label;
        }

        @Override
        public String toString() {
            return label;
        }
    }

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Lob
    @Column(nullable = false)
    private byte[] data;

    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 20)
    private Role role = Role.PIECE;

    @Column(length = 200)
    private String caption = "";

    @Column(name = "added_at", nullable = false)
    private Instant addedAt = Instant.now();

    protected ProjectPhoto() {
        // requis par JPA
    }

    public ProjectPhoto(byte[] data, Role role, String caption) {
        this.data = data;
        this.role = role;
        this.caption = caption == null ? "" : caption;
    }

    public Long getId() {
        return id;
    }

    public byte[] getData() {
        return data;
    }

    public Role getRole() {
        return role;
    }

    public void setRole(Role role) {
        this.role = role;
    }

    public String getCaption() {
        return caption;
    }

    public void setCaption(String caption) {
        this.caption = caption;
    }

    public Instant getAddedAt() {
        return addedAt;
    }
}
