package be.asmolabs.palettier.domain.project

import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint

internal fun paint(
    id: Long,
    name: String,
    hex: String,
    inStock: Boolean = true,
    brand: String = "W&N",
) = Paint(
    id = id, brand = brand, name = name, hexColor = hex, inStock = inStock,
    opacity = Opacity.SEMI_OPAQUE, dryingClass = DryingClass.MEDIUM, tintingStrength = 0.8,
)

internal fun layer(role: String, hex: String = "#C98F72", technique: String = "Glacis") =
    ProjectLayer(role = role, targetHex = hex, technique = technique)

internal fun zone(name: String, vararg layers: ProjectLayer) =
    ProjectZone(name = name, material = "Peau", layers = layers.toList())

internal fun project(name: String, vararg zones: ProjectZone) =
    Project(id = 1, name = name, subject = "Buste", zones = zones.toList())
