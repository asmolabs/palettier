//! Mixes laid out on the palette and followed on the Today screen.

use crate::state::AppServices;
use chrono::Utc;
use palettier_domain::drying::Workshop;
use palettier_domain::error::DomainResult;
use palettier_domain::paint::{DryingClass, PaintSystem};
use palettier_domain::palette::PaletteMix;
use palettier_domain::port::PaletteMixRepository;

/// A mix about to be laid on the palette.
#[derive(Debug, Clone)]
pub struct NewMix {
    pub name: String,
    pub hex_color: String,
    pub recipe: String,
    pub drying_class: DryingClass,
    pub system: PaintSystem,
    pub workshop: Workshop,
}

/// Lays a mix on the palette now and returns every mix followed. A colour typed without
/// its `#` gets one. Returns `None` when the name is blank (nothing is saved).
pub async fn place_mix(services: &AppServices, mix: NewMix) -> DomainResult<Option<Vec<PaletteMix>>> {
    let name = mix.name.trim().to_string();
    if name.is_empty() {
        return Ok(None);
    }
    let hex = mix.hex_color.trim();
    let mix = PaletteMix {
        id: None,
        name,
        hex_color: if hex.starts_with('#') {
            hex.to_string()
        } else {
            format!("#{hex}")
        },
        recipe: mix.recipe.trim().to_string(),
        drying_class: mix.drying_class,
        system: mix.system,
        mixed_at: Utc::now(),
        workshop: mix.workshop,
    };
    services.mixes.save(&mix).await?;
    services.mixes.all().await.map(Some)
}

/// Takes a mix off the palette and returns the mixes left.
pub async fn remove_mix(services: &AppServices, id: i64) -> DomainResult<Vec<PaletteMix>> {
    services.mixes.delete(id).await?;
    services.mixes.all().await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mix(name: &str, hex: &str, system: PaintSystem) -> NewMix {
        NewMix {
            name: name.into(),
            hex_color: hex.into(),
            recipe: " 2 Ocre + 1 Blanc ".into(),
            drying_class: DryingClass::Slow,
            system,
            workshop: Workshop::default(),
        }
    }

    #[tokio::test]
    async fn places_and_removes_a_mix() {
        let svc = AppServices::in_memory().await.unwrap();
        let all = place_mix(&svc, mix(" Chair ", "8C6248", PaintSystem::Acrylic))
            .await
            .unwrap()
            .unwrap();
        let placed = all.iter().find(|m| m.name == "Chair").unwrap();
        assert_eq!(placed.hex_color, "#8C6248");
        assert_eq!(placed.recipe, "2 Ocre + 1 Blanc");
        assert_eq!(placed.system, PaintSystem::Acrylic);

        let left = remove_mix(&svc, placed.id.unwrap()).await.unwrap();
        assert!(left.iter().all(|m| m.name != "Chair"));
    }

    #[tokio::test]
    async fn a_mix_without_a_name_is_not_placed() {
        let svc = AppServices::in_memory().await.unwrap();
        assert!(place_mix(&svc, mix(" ", "#123456", PaintSystem::Oil)).await.unwrap().is_none());
    }
}
