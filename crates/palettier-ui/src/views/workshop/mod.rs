//! The workshop screens: Today, Projects and Palettes.

mod palettes;
mod project_detail;
mod projects;
mod today;
mod zones;

pub use palettes::PalettesView;
pub use projects::ProjectsView;
pub use today::TodayView;

use palettier_domain::i18n::Locale;
use std::time::Duration;

pub fn format_duration(d: Duration, locale: Locale) -> String {
    let total_mins = d.as_secs() / 60;
    if total_mins == 0 {
        locale.tr("Prêt maintenant", "Ready now").to_string()
    } else if total_mins < 60 {
        format!("{total_mins} min")
    } else {
        let hours = total_mins / 60;
        let mins = total_mins % 60;
        if hours < 48 {
            if mins == 0 {
                format!("{hours} h")
            } else {
                format!("{hours} h {mins:02}")
            }
        } else {
            let days = hours / 24;
            let rem_h = hours % 24;
            let d_unit = locale.tr("j", "d");
            if rem_h == 0 {
                format!("{days} {d_unit}")
            } else {
                format!("{days} {d_unit} {rem_h} h")
            }
        }
    }
}
