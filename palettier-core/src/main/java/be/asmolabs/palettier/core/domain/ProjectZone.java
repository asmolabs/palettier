package be.asmolabs.palettier.core.domain;

import jakarta.persistence.CascadeType;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.FetchType;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.OneToMany;
import jakarta.persistence.OrderColumn;
import jakarta.persistence.Table;
import java.util.ArrayList;
import java.util.List;

/** Une partie du sujet dans un projet : le visage, la cape, le ceinturon. */
@Entity
@Table(name = "project_zone")
public class ProjectZone {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(nullable = false)
    private String name;

    @Column
    private String material = "";

    @Column(length = 1000)
    private String note = "";

    @OneToMany(cascade = CascadeType.ALL, orphanRemoval = true, fetch = FetchType.EAGER)
    @JoinColumn(name = "zone_id", nullable = false)
    @OrderColumn(name = "layer_position")
    private List<ProjectLayer> layers = new ArrayList<>();

    protected ProjectZone() {
        // requis par JPA
    }

    public ProjectZone(String name, String material, String note) {
        this.name = name;
        this.material = material == null ? "" : material;
        this.note = note == null ? "" : note;
    }

    public ProjectZone addLayer(ProjectLayer layer) {
        layers.add(layer);
        return this;
    }

    public Long getId() {
        return id;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getMaterial() {
        return material;
    }

    public void setMaterial(String material) {
        this.material = material;
    }

    public String getNote() {
        return note;
    }

    public void setNote(String note) {
        this.note = note;
    }

    public List<ProjectLayer> getLayers() {
        return layers;
    }
}
