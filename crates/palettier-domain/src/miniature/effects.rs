//! Weathering and special effects: washes, streaks, rust, blood, dry pigments, vehicle filters.

use super::*;

/// Weathering & Grimdark effects library for miniature modellers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeatheringEffect {
    StreakingGrime,
    FreshRust,
    CrustyOldRust,
    VerdigrisPatina,
    DustDeposit,
    DryMud,
    FreshFuelOil,
}

impl WeatheringEffect {
    pub const ALL: [WeatheringEffect; 7] = [
        WeatheringEffect::StreakingGrime,
        WeatheringEffect::FreshRust,
        WeatheringEffect::CrustyOldRust,
        WeatheringEffect::VerdigrisPatina,
        WeatheringEffect::DustDeposit,
        WeatheringEffect::DryMud,
        WeatheringEffect::FreshFuelOil,
    ];

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::StreakingGrime, Locale::Fr) => "Streaking Grime (Coulures de crasse & pluie)",
            (Self::StreakingGrime, Locale::En) => "Streaking Grime (Rain & Grim streaks)",
            (Self::FreshRust, Locale::Fr) => "Rouille vive / Fraîche",
            (Self::FreshRust, Locale::En) => "Fresh Bright Rust",
            (Self::CrustyOldRust, Locale::Fr) => "Rouille ancienne & encroûtée",
            (Self::CrustyOldRust, Locale::En) => "Old Crusty Rust",
            (Self::VerdigrisPatina, Locale::Fr) => "Vert-de-gris / Patine de bronze & cuivre",
            (Self::VerdigrisPatina, Locale::En) => "Verdigris / Bronze & Copper patina",
            (Self::DustDeposit, Locale::Fr) => "Dépôt de poussière sèche d'atelier",
            (Self::DustDeposit, Locale::En) => "Dry Dust Deposit",
            (Self::DryMud, Locale::Fr) => "Boue séchée / Éclaboussures",
            (Self::DryMud, Locale::En) => "Dry Mud & Splatter",
            (Self::FreshFuelOil, Locale::Fr) => "Fuite de carburant & graisse fraîche",
            (Self::FreshFuelOil, Locale::En) => "Fresh Fuel & Oil Stains",
        }
    }

    pub fn hex_color(&self) -> &'static str {
        match self {
            Self::StreakingGrime => "#4A3E2D",
            Self::FreshRust => "#C85A17",
            Self::CrustyOldRust => "#652C16",
            Self::VerdigrisPatina => "#40A892",
            Self::DustDeposit => "#C2B294",
            Self::DryMud => "#594A36",
            Self::FreshFuelOil => "#1B1713",
        }
    }

    pub fn oil_recipe_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::StreakingGrime, Locale::Fr) => "3 parts Terre d'Ombre Naturelle (PBr7) + 1 part Noir de Vigne/Ivoire (PBk9) + diluant minéral sans odeur",
            (Self::StreakingGrime, Locale::En) => "3 parts Raw Umber (PBr7) + 1 part Vine/Ivory Black (PBk9) + odorless mineral spirits",
            (Self::FreshRust, Locale::Fr) => "3 parts Terre de Sienne Brûlée (PBr7/PR101) + 1 part Orange de Cadmium (PO20/PY35)",
            (Self::FreshRust, Locale::En) => "3 parts Burnt Sienna (PBr7/PR101) + 1 part Cadmium Orange (PO20/PY35)",
            (Self::CrustyOldRust, Locale::Fr) => "4 parts Oxyde Rouge de Fer (PR101) + 1 part Terre d'Ombre Brûlée (PBr7)",
            (Self::CrustyOldRust, Locale::En) => "4 parts Red Iron Oxide (PR101) + 1 part Burnt Umber (PBr7)",
            (Self::VerdigrisPatina, Locale::Fr) => "3 parts Oxyde de Chrome Vert (PG17) + 1 part Bleu Phtalo (PB15) + 4 parts Blanc de Titane (PW6)",
            (Self::VerdigrisPatina, Locale::En) => "3 parts Chromium Oxide Green (PG17) + 1 part Phthalo Blue (PB15) + 4 parts Titanium White (PW6)",
            (Self::DustDeposit, Locale::Fr) => "3 parts Ocre Jaune (PY43) + 1 part Blanc de Titane (PW6) + 1 part Terre d'Ombre Naturelle",
            (Self::DustDeposit, Locale::En) => "3 parts Yellow Ochre (PY43) + 1 part Titanium White (PW6) + 1 part Raw Umber",
            (Self::DryMud, Locale::Fr) => "4 parts Terre d'Ombre Naturelle + 2 parts Ocre Jaune + 1 part Noir d'Ivoire",
            (Self::DryMud, Locale::En) => "4 parts Raw Umber + 2 parts Yellow Ochre + 1 part Ivory Black",
            (Self::FreshFuelOil, Locale::Fr) => "4 parts Noir d'Ivoire (PBk9) + 1 part Terre de Sienne Brûlée + vernis brillant pur",
            (Self::FreshFuelOil, Locale::En) => "4 parts Ivory Black (PBk9) + 1 part Burnt Sienna + pure gloss varnish",
        }
    }

    pub fn application_technique_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::StreakingGrime, Locale::Fr) => "Poser en micro-points sur les rivets, puis tirer verticalement au pinceau biseauté humecté d'essence sans odeur.",
            (Self::StreakingGrime, Locale::En) => "Apply dot on rivets, then feather vertically downward with an odorless spirit-dampened brush.",
            (Self::FreshRust, Locale::Fr) => "Appliquer au pinceau fin dans les creux des plaques métalliques ou tamponner à l'éponge blister.",
            (Self::FreshRust, Locale::En) => "Apply with fine brush into armor joints or stipple using blister sponge.",
            (Self::CrustyOldRust, Locale::Fr) => "Base texturée sous la rouille fraîche pour donner du relief aux plaques corrodées.",
            (Self::CrustyOldRust, Locale::En) => "Use as darker textured base beneath fresh bright rust spots.",
            (Self::VerdigrisPatina, Locale::Fr) => "Diluer en jus très liquide (1:8), laisser filer par capillarité dans les creux de cuivre/bronze.",
            (Self::VerdigrisPatina, Locale::En) => "Dilute to wash consistency (1:8), let flow by capillary action into bronze/copper recesses.",
            (Self::DustDeposit, Locale::Fr) => "Brosser à sec ou en voile ultra-mat sur le bas des manteaux, chenilles et socles.",
            (Self::DustDeposit, Locale::En) => "Drybrush or feather matte wash onto cloak hems, tank tracks, and bases.",
            (Self::DryMud, Locale::Fr) => "Projeter délicatement avec un cure-dent ou un pinceau à poils durs sur les bas de caisse.",
            (Self::DryMud, Locale::En) => "Flick using stiff brush or toothpick onto vehicle skirts and boots.",
            (Self::FreshFuelOil, Locale::Fr) => "Poser pur sur les bouchons de réservoir et vérins hydrauliques pour garder le brillant.",
            (Self::FreshFuelOil, Locale::En) => "Apply pure around fuel caps and pistons to preserve glossy wet appearance.",
        }
    }
}

/// Pin wash & reductive technique configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinWashConfig {
    pub ratio_oil: u32,
    pub ratio_solvent: u32,
    pub min_cleaning_window_mins: u32,
    pub max_cleaning_window_mins: u32,
}

impl PinWashConfig {
    /// Armor panel pin wash (optimal capillary flow in panel lines and around rivets).
    pub fn armor_pin_wash() -> Self {
        Self {
            ratio_oil: 1,
            ratio_solvent: 6,
            min_cleaning_window_mins: 15,
            max_cleaning_window_mins: 90,
        }
    }

    /// Deep recess shadow wash.
    pub fn deep_recess_wash() -> Self {
        Self {
            ratio_oil: 1,
            ratio_solvent: 3,
            min_cleaning_window_mins: 20,
            max_cleaning_window_mins: 120,
        }
    }

    /// Overall armor tint filter.
    pub fn overall_filter() -> Self {
        Self {
            ratio_oil: 1,
            ratio_solvent: 12,
            min_cleaning_window_mins: 10,
            max_cleaning_window_mins: 60,
        }
    }

    pub fn capillary_flow_desc(&self, locale: Locale) -> &'static str {
        if self.ratio_solvent >= 10 {
            locale.tr("Capillarité ultra-haute (voile teinté sans taches)", "Ultra-high capillary flow (glaze filter)")
        } else if self.ratio_solvent >= 5 {
            locale.tr("Capillarité optimale (file instantanément le long des arêtes)", "Optimal capillary flow (snaps along panel lines)")
        } else {
            locale.tr("Consistance semi-dense (lavis concentré de creux)", "Semi-dense wash (concentrated recess shade)")
        }
    }
}

/// Special effects recipes & textured applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialEffectGore {
    FreshGore,
    CoagulatedBlood,
    FrostIce,
    XenoSlime,
    GemLens,
}

impl SpecialEffectGore {
    pub const ALL: [SpecialEffectGore; 5] = [
        SpecialEffectGore::FreshGore,
        SpecialEffectGore::CoagulatedBlood,
        SpecialEffectGore::FrostIce,
        SpecialEffectGore::XenoSlime,
        SpecialEffectGore::GemLens,
    ];

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshGore, Locale::Fr) => "Sang Frais & Éclaboussures Récentes",
            (Self::FreshGore, Locale::En) => "Fresh Arterial Gore & Splatter",
            (Self::CoagulatedBlood, Locale::Fr) => "Sang Coagulé & Plaies Sèches",
            (Self::CoagulatedBlood, Locale::En) => "Coagulated Blood & Crusted Wounds",
            (Self::FrostIce, Locale::Fr) => "Givre & Glace de Socle",
            (Self::FrostIce, Locale::En) => "Frostbite & Tundra Ice",
            (Self::XenoSlime, Locale::Fr) => "Bave Xéno & Acide Toxique",
            (Self::XenoSlime, Locale::En) => "Xeno Slime & Toxic Acid",
            (Self::GemLens, Locale::Fr) => "Gemmes & Optiques Réfléchissantes",
            (Self::GemLens, Locale::En) => "Reflective Gemstones & Lenses",
        }
    }

    pub fn hex_color(&self) -> &'static str {
        match self {
            Self::FreshGore => "#8F0808",
            Self::CoagulatedBlood => "#3F0C0C",
            Self::FrostIce => "#D0F0FD",
            Self::XenoSlime => "#86D400",
            Self::GemLens => "#00BCD4",
        }
    }

    pub fn recipe_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshGore, Locale::Fr) => "Rouge Alizarine/Phtalo (PR177) + pointe Terre d'Ombre Brûlée (PBr7) + vernis brillant pur",
            (Self::FreshGore, Locale::En) => "Alizarin/Phthalo Red (PR177) + hint of Burnt Umber (PBr7) + pure gloss varnish",
            (Self::CoagulatedBlood, Locale::Fr) => "Oxyde de Fer Rouge (PR101) + Noir d'Ivoire (PBk9) + fini satiné",
            (Self::CoagulatedBlood, Locale::En) => "Red Iron Oxide (PR101) + Ivory Black (PBk9) + satin finish",
            (Self::FrostIce, Locale::Fr) => "Blanc de Titane (PW6) + voile transparent Bleu Phtalo (PB15) + vernis satin mat",
            (Self::FrostIce, Locale::En) => "Titanium White (PW6) + transparent glaze Phthalo Blue (PB15) + satin-matte finish",
            (Self::XenoSlime, Locale::Fr) => "Jaune de Cadmium (PY35) + Bleu Phtalo (PB15) + vernis brillant pur étiré",
            (Self::XenoSlime, Locale::En) => "Cadmium Yellow (PY35) + Phthalo Blue (PB15) + pure high-gloss varnish",
            (Self::GemLens, Locale::Fr) => "Dégradé inverse : Noir en haut à 11h vers Cyan lumineux en bas à 5h + point spéculaire blanc pur à 11h",
            (Self::GemLens, Locale::En) => "Inverted gradient: Black at 11 o'clock to Bright Cyan at 5 o'clock + pure white dot at 11 o'clock",
        }
    }

    pub fn technique_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshGore, Locale::Fr) => "Projeter avec une brosse à dents ou souffler à l'aérographe sur la pointe d'un pinceau chargé.",
            (Self::FreshGore, Locale::En) => "Flick using stiff toothbrush or airbrush pulse against loaded brush bristles.",
            (Self::CoagulatedBlood, Locale::Fr) => "Tamponner à la mousse de blister dans les creux des armures et sur les bords des lames.",
            (Self::CoagulatedBlood, Locale::En) => "Stipple with blister sponge inside armor recesses and along blade cutting edges.",
            (Self::FrostIce, Locale::Fr) => "Brosser à sec très léger sur le bord des manteaux et les pointes des touffes d'herbe.",
            (Self::FrostIce, Locale::En) => "Ultra-light drybrush on cloak hems and tips of tufts.",
            (Self::XenoSlime, Locale::Fr) => "Étendre des filaments de colle polymère (UHU) puis teinter au jus d'huile vert translucide.",
            (Self::XenoSlime, Locale::En) => "String out clear polymer glue strands (UHU), then glaze with translucent green oil tint.",
            (Self::GemLens, Locale::Fr) => "Poser une goutte de vernis brillant après séchage complet pour donner une courbure vitrée réelle.",
            (Self::GemLens, Locale::En) => "Place a domed droplet of gloss varnish after thorough curing for authentic 3D refraction.",
        }
    }
}

/// Dry weathering pigment reference with wet-shift simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DryPigmentEffect {
    BurntUmberDust,
    ElysiumDesertDust,
    SootBlack,
    FreshRustPowder,
    IndustrialAsh,
}

impl DryPigmentEffect {
    pub const ALL: [Self; 5] = [
        Self::BurntUmberDust,
        Self::ElysiumDesertDust,
        Self::SootBlack,
        Self::FreshRustPowder,
        Self::IndustrialAsh,
    ];

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::BurntUmberDust, Locale::Fr) => "Poudre Terre d'Ombre Brûlée",
            (Self::BurntUmberDust, Locale::En) => "Burnt Umber Soil Dust",
            (Self::ElysiumDesertDust, Locale::Fr) => "Poussière Désertique / Sable",
            (Self::ElysiumDesertDust, Locale::En) => "Desert Dust / Dune Sand",
            (Self::SootBlack, Locale::Fr) => "Suie Noire d'Échappement & Canon",
            (Self::SootBlack, Locale::En) => "Exhaust & Barrel Soot Black",
            (Self::FreshRustPowder, Locale::Fr) => "Pigment Rouille Calcinée Vive",
            (Self::FreshRustPowder, Locale::En) => "Vivid Rust Powder",
            (Self::IndustrialAsh, Locale::Fr) => "Cendre & Gravats Urbains",
            (Self::IndustrialAsh, Locale::En) => "Urban Ash & Rubble",
        }
    }

    /// Dry powder appearance (lighter, chalky, matte).
    pub fn dry_hex(&self) -> &'static str {
        match self {
            Self::BurntUmberDust => "#7D583F",
            Self::ElysiumDesertDust => "#DEC6A5",
            Self::SootBlack => "#303030",
            Self::FreshRustPowder => "#E0682B",
            Self::IndustrialAsh => "#A8A5A0",
        }
    }

    /// Darkened appearance once wetted or fixed with solvent / binder.
    pub fn fixed_hex(&self) -> &'static str {
        match self {
            Self::BurntUmberDust => "#3E2718",
            Self::ElysiumDesertDust => "#9F7E54",
            Self::SootBlack => "#111111",
            Self::FreshRustPowder => "#8F3810",
            Self::IndustrialAsh => "#575450",
        }
    }

    pub fn application_tip_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::BurntUmberDust, Locale::Fr) => "Déposer à sec dans les chenilles, puis capillariser une goutte d'essence sans odeur pour fixer sans écraser le volume.",
            (Self::BurntUmberDust, Locale::En) => "Deposit dry onto tracks, then touch with a drop of odorless spirit to wick and fix without flattening texture.",
            (Self::ElysiumDesertDust, Locale::Fr) => "Brosser à sec sur le bas des capes et des bottes. Très volatile : ne pas vernir à l'aérographe de trop près pour ne pas souffler la poudre.",
            (Self::ElysiumDesertDust, Locale::En) => "Dust dry onto bottom of cloaks and boots. Highly volatile: do not airbrush varnish too close to avoid blowing powder away.",
            (Self::SootBlack, Locale::Fr) => "Appliquer pur au pinceau sec ébouriffé sur les sorties de réacteurs, silencieux et canons. Ne pas fixer pour conserver le noir velouté ultra-mat.",
            (Self::SootBlack, Locale::En) => "Apply pure with a frayed drybrush onto engine exhausts, muzzles, and cannons. Leave unfixed for ultra-matte velvety soot.",
            (Self::FreshRustPowder, Locale::Fr) => "Saupoudrer sur un jus d'huile encore frais pour créer une texture granuleuse en relief d'oxydation réelle.",
            (Self::FreshRustPowder, Locale::En) => "Stipple onto still-wet oil wash to instantly create real textured corrosion relief.",
            (Self::IndustrialAsh, Locale::Fr) => "Mélanger avec du plâtre ou du pigment sombre pour soclage urbain et décombres d'usines.",
            (Self::IndustrialAsh, Locale::En) => "Combine with fine plaster or dark pigments for urban warfare basing and rubble.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleEffectKind {
    StarshipFilth,
    EngineGrime,
    DarkRust,
    IndustrialEarth,
    TrackWash,
    Rainmarks,
    FreshEngineOil,
}

impl VehicleEffectKind {
    pub const ALL: [VehicleEffectKind; 7] = [
        VehicleEffectKind::StarshipFilth,
        VehicleEffectKind::EngineGrime,
        VehicleEffectKind::DarkRust,
        VehicleEffectKind::IndustrialEarth,
        VehicleEffectKind::TrackWash,
        VehicleEffectKind::Rainmarks,
        VehicleEffectKind::FreshEngineOil,
    ];

    pub fn commercial_name(&self) -> &'static str {
        match self {
            Self::StarshipFilth => "Starship Filth (Abteilung 502 ABT160)",
            Self::EngineGrime => "Engine Grime (AK-026 / ABT100)",
            Self::DarkRust => "Dark Rust (Abteilung 502 ABT070)",
            Self::IndustrialEarth => "Industrial Earth (Ammo Mig A.MIG-1407)",
            Self::TrackWash => "Track Wash (AK-083 / ABT094)",
            Self::Rainmarks => "Rainmarks / Streaking Dust (AK-012)",
            Self::FreshEngineOil => "Fresh Engine Oil (Ammo Mig A.MIG-1408)",
        }
    }

    pub fn target_hex(&self) -> &'static str {
        match self {
            Self::StarshipFilth => "#3F3A36",
            Self::EngineGrime => "#2D261E",
            Self::DarkRust => "#652D19",
            Self::IndustrialEarth => "#886B52",
            Self::TrackWash => "#241D1A",
            Self::Rainmarks => "#D6CCA9",
            Self::FreshEngineOil => "#120F0C",
        }
    }

    pub fn category_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::StarshipFilth, Locale::Fr) => "Salissure Panneaux Blindés & Sci-Fi",
            (Self::StarshipFilth, Locale::En) => "Armor & Sci-Fi Panel Grime",
            (Self::EngineGrime, Locale::Fr) => "Moteur, Échappements & Graisse",
            (Self::EngineGrime, Locale::En) => "Engine, Exhausts & Grease",
            (Self::DarkRust, Locale::Fr) => "Rouille Sombre & Oxydation Métal",
            (Self::DarkRust, Locale::En) => "Dark Rust & Metal Oxidation",
            (Self::IndustrialEarth, Locale::Fr) => "Poussière de Terre Industrielle",
            (Self::IndustrialEarth, Locale::En) => "Industrial Earth & Dust",
            (Self::TrackWash, Locale::Fr) => "Chenilles & Barbotins de Chars",
            (Self::TrackWash, Locale::En) => "Tank Tracks & Sprockets Wash",
            (Self::Rainmarks, Locale::Fr) => "Coulures de Pluie & Poussière Verticale",
            (Self::Rainmarks, Locale::En) => "Rainmarks & Vertical Dust Streaks",
            (Self::FreshEngineOil, Locale::Fr) => "Fuite d'Huile de Moteur Fraîche",
            (Self::FreshEngineOil, Locale::En) => "Fresh Wet Engine Oil Leak",
        }
    }

    pub fn artist_oil_formula_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::StarshipFilth, Locale::Fr) => "45% Terre d'Ombre Brûlée + 45% Noir d'Ivoire + 10% Blanc de Titane",
            (Self::StarshipFilth, Locale::En) => "45% Burnt Umber + 45% Ivory Black + 10% Titanium White",
            (Self::EngineGrime, Locale::Fr) => "60% Terre d'Ombre Brûlée + 30% Vert Olive (ou Viridian + pointe d'Ocre) + 10% Noir",
            (Self::EngineGrime, Locale::En) => "60% Burnt Umber + 30% Olive Green (or Viridian + touch of Ochre) + 10% Black",
            (Self::DarkRust, Locale::Fr) => "70% Terre de Sienne Brûlée + 20% Rouge de Cadmium + 10% Terre d'Ombre Brûlée",
            (Self::DarkRust, Locale::En) => "70% Burnt Sienna + 20% Cadmium Red + 10% Burnt Umber",
            (Self::IndustrialEarth, Locale::Fr) => "60% Ocre Jaune + 30% Terre d'Ombre Brûlée + 10% Blanc de Titane",
            (Self::IndustrialEarth, Locale::En) => "60% Yellow Ochre + 30% Burnt Umber + 10% Titanium White",
            (Self::TrackWash, Locale::Fr) => "75% Terre d'Ombre Brûlée + 20% Noir d'Ivoire + 5% Bleu Outremer",
            (Self::TrackWash, Locale::En) => "75% Burnt Umber + 20% Ivory Black + 5% Ultramarine Blue",
            (Self::Rainmarks, Locale::Fr) => "70% Blanc de Titane + 25% Ocre Jaune + 5% Terre d'Ombre Naturelle",
            (Self::Rainmarks, Locale::En) => "70% Titanium White + 25% Yellow Ochre + 5% Raw Umber",
            (Self::FreshEngineOil, Locale::Fr) => "80% Noir d'Ivoire + 15% Terre d'Ombre Brûlée + 5% Laque de Garance (brillant)",
            (Self::FreshEngineOil, Locale::En) => "80% Ivory Black + 15% Burnt Umber + 5% Madder Lake (gloss finish)",
        }
    }

    pub fn solvent_and_dilution_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshEngineOil, Locale::Fr) => "1 part de pâte pure + 2 gouttes de Vernis Brillant / Liquin. Ne PAS diluer au solvant inodore pour conserver l'effet liquide sirupeux.",
            (Self::FreshEngineOil, Locale::En) => "1 part pure paste + 2 drops Gloss Varnish / Liquin. Do NOT thin with solvent to maintain wet syrupy gloss.",
            (Self::Rainmarks, Locale::Fr) => "1 part de pâte pour 12 gouttes d'essence inodore (Sansodor/Gamsol). Déposer en micro-points puis étirer verticalement au pinceau plat propre imbibé de solvant.",
            (Self::Rainmarks, Locale::En) => "1 part paste to 12 drops odorless mineral spirit. Dot sparingly then streak vertically with a clean flat brush damp with solvent.",
            (_, Locale::Fr) => "1 part de pâte pour 8 à 10 gouttes de Sansodor / Gamsol (solvant inodore doux pour protéger le vernis acrylique préalable).",
            (_, Locale::En) => "1 part paste to 8-10 drops Sansodor / Gamsol (gentle odorless spirit to preserve underlying acrylic clear coat).",
        }
    }

    pub fn finish_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshEngineOil, Locale::Fr) => "✨ Brillant miroir humide (Gloss)",
            (Self::FreshEngineOil, Locale::En) => "✨ Gloss wet look",
            (Self::Rainmarks | Self::IndustrialEarth, Locale::Fr) => "🌫️ Ultra-Mat poussiéreux",
            (Self::Rainmarks | Self::IndustrialEarth, Locale::En) => "🌫️ Dead matte chalky",
            (_, Locale::Fr) => "🛡️ Satiné naturel",
            (_, Locale::En) => "🛡️ Natural satin",
        }
    }
}
