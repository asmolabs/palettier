//! Monochrome `16×16` SVG icons for shell navigation and UI controls.

use crate::state::Section;
use dioxus::prelude::*;

/// Shared `16×16` stroke SVG wrapper preserving `currentColor` inheritance.
#[component]
fn Svg16(class: Option<String>, children: Element) -> Element {
    let cls = class.unwrap_or_else(|| "ui-icon".to_string());
    rsx! {
        svg {
            class: "{cls}",
            width: "16",
            height: "16",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            {children}
        }
    }
}

/// Magnifying glass icon for search and command triggers.
#[component]
pub fn IconSearch(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            circle { cx: "11", cy: "11", r: "7" }
            path { d: "m21 21-4.3-4.3" }
        }
    }
}

/// Sun icon for Light theme selection.
#[component]
pub fn IconSun(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            circle { cx: "12", cy: "12", r: "4" }
            path { d: "M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" }
        }
    }
}

/// Crescent moon icon for Dark theme selection.
#[component]
pub fn IconMoon(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            path { d: "M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" }
        }
    }
}

/// Split-circle / contrast icon for Neutral Graphite theme selection.
#[component]
pub fn IconContrast(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            circle { cx: "12", cy: "12", r: "9" }
            path { d: "M12 3v18M12 8l5-3M12 13l7-4M12 18l5-3" }
        }
    }
}

/// Crosshair / HUD icon for the compact bench mode toggle.
#[component]
pub fn IconTarget(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            circle { cx: "12", cy: "12", r: "9" }
            circle { cx: "12", cy: "12", r: "3" }
            path { d: "M12 2v3M12 19v3M2 12h3M19 12h3" }
        }
    }
}

/// Globe icon for language toggle.
#[component]
pub fn IconGlobe(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            circle { cx: "12", cy: "12", r: "9" }
            path { d: "M3 12h18M12 3a14.5 14.5 0 0 0 0 18M12 3a14.5 14.5 0 0 1 0 18" }
        }
    }
}

/// Gear icon for Settings and calibration.
#[component]
pub fn IconSettings(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            circle { cx: "12", cy: "12", r: "3" }
            path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z" }
        }
    }
}

/// Left chevron icon for sidebar collapse or back actions.
#[component]
pub fn IconChevronLeft(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            path { d: "m15 18-6-6 6-6" }
        }
    }
}

/// Right chevron icon for sidebar expand or step-forward actions.
#[component]
pub fn IconChevronRight(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            path { d: "m9 18 6-6-6-6" }
        }
    }
}

/// Close (`×`) icon for drawers, modals, and clear buttons.
#[component]
pub fn IconClose(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            path { d: "M18 6 6 18M6 6l12 12" }
        }
    }
}

/// Plus icon for creation drawers and add buttons.
#[component]
pub fn IconPlus(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            path { d: "M12 5v14M5 12h14" }
        }
    }
}

/// Checkmark circle icon for success toast notices.
#[component]
pub fn IconCheck(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            circle { cx: "12", cy: "12", r: "9" }
            path { d: "m9 12 2 2 4-4" }
        }
    }
}

/// Alert circle icon for warning or error toast notices.
#[component]
pub fn IconAlert(class: Option<String>) -> Element {
    rsx! {
        Svg16 { class,
            circle { cx: "12", cy: "12", r: "9" }
            path { d: "M12 8v4M12 16h.01" }
        }
    }
}

/// Crisp `16×16` SVG icon mapped to each application [`Section`].
#[component]
pub fn SectionIcon(section: Section, class: Option<String>) -> Element {
    match section {
        Section::Today => rsx! {
            Svg16 { class,
                circle { cx: "12", cy: "12", r: "4" }
                path { d: "M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2" }
            }
        },
        Section::Projects => rsx! {
            Svg16 { class,
                path { d: "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z" }
            }
        },
        Section::Palettes => rsx! {
            Svg16 { class,
                circle { cx: "13.5", cy: "6.5", r: ".5" }
                circle { cx: "17.5", cy: "10.5", r: ".5" }
                circle { cx: "8.5", cy: "7.5", r: ".5" }
                circle { cx: "6.5", cy: "12.5", r: ".5" }
                path { d: "M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2Z" }
            }
        },
        Section::Mixer => rsx! {
            Svg16 { class,
                path { d: "M10 2v7.31M14 9.3V2M8.5 2h7M14 9.3a6.5 6.5 0 1 1-4 0M5.52 16h12.96" }
            }
        },
        Section::Eyedropper => rsx! {
            Svg16 { class,
                path { d: "m2 22 1-1h3l9-9M3 21v-3l9-9M15 6l3.4-3.4a2.1 2.1 0 1 1 3 3L18 9l.4.4a2.1 2.1 0 1 1-3 3l-3.8-3.8a2.1 2.1 0 1 1 3-3l.4.4Z" }
            }
        },
        Section::Assistant => rsx! {
            Svg16 { class,
                path { d: "M9.937 15.5A2 2 0 0 0 8.5 14.063l-6.135-1.582a.5.5 0 0 1 0-.962L8.5 9.936A2 2 0 0 0 9.937 8.5l1.582-6.135a.5.5 0 0 1 .963 0L14.063 8.5A2 2 0 0 0 15.5 9.937l6.135 1.581a.5.5 0 0 1 0 .964L15.5 14.063a2 2 0 0 0-1.437 1.437l-1.582 6.135a.5.5 0 0 1-.963 0Z" }
            }
        },
        Section::Miniature => rsx! {
            Svg16 { class,
                path { d: "M14.5 17.5 3 6V3h3l11.5 11.5M13 19l6-6M16 16l4 4M19 21l2-2" }
            }
        },
        Section::Catalog => rsx! {
            Svg16 { class,
                path { d: "M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20" }
            }
        },
        Section::Drying => rsx! {
            Svg16 { class,
                circle { cx: "12", cy: "12", r: "9" }
                path { d: "M12 7v5l3 3" }
            }
        },
        Section::Recipes => rsx! {
            Svg16 { class,
                path { d: "M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" }
            }
        },
        Section::Settings => rsx! {
            IconSettings { class }
        },
    }
}
