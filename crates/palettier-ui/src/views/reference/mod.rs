//! The reference screens: the tube catalog, the drying calculator and the recipes.

mod catalog;
mod catalog_sheet;
mod catalog_tubes;
mod drying;
mod drying_cards;
mod recipe_detail;
mod recipes;

pub use catalog::CatalogView;
pub use drying::DryingView;
pub use recipes::RecipesView;

use palettier_domain::i18n::Locale;
use std::time::Duration;

fn format_duration(d: Duration, locale: Locale) -> String {
    let total_mins = d.as_secs() / 60;
    if total_mins == 0 {
        return locale.tr("immédiat", "immediate").to_string();
    }
    let days = total_mins / (60 * 24);
    let hours = (total_mins % (60 * 24)) / 60;
    let mins = total_mins % 60;

    let (m_unit, d_unit) = match locale {
        Locale::Fr => ("mois", "j"),
        Locale::En => ("mo", "d"),
    };

    if days > 30 {
        let months = days / 30;
        let rem_days = days % 30;
        if rem_days > 0 {
            format!("{months} {m_unit} {rem_days} {d_unit}")
        } else {
            format!("{months} {m_unit}")
        }
    } else if days > 0 {
        if hours > 0 {
            format!("{days} {d_unit} {hours} h")
        } else {
            format!("{days} {d_unit}")
        }
    } else if hours > 0 {
        if mins > 0 {
            format!("{hours} h {mins:02} min")
        } else {
            format!("{hours} h")
        }
    } else {
        format!("{mins} min")
    }
}
