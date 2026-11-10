use crate::error::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::fmt;

/// sRGB color with each component in the `[0, 1]` range.
///
/// Components are clamped to the interval rather than rejected, matching the
/// compact constructor on the Java/Kotlin side: linear-light / sRGB conversions
/// can produce values that overshoot by a billionth.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rgb {
    r: f64,
    g: f64,
    b: f64,
}

impl Rgb {
    pub const REFERENCE_WHITE: Rgb = Rgb {
        r: 0xF7 as f64 / 255.0,
        g: 0xF5 as f64 / 255.0,
        b: 0xF0 as f64 / 255.0,
    };

    #[inline]
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
        }
    }

    #[inline]
    pub fn r(&self) -> f64 {
        self.r
    }

    #[inline]
    pub fn g(&self) -> f64 {
        self.g
    }

    #[inline]
    pub fn b(&self) -> f64 {
        self.b
    }

    #[inline]
    pub fn components(&self) -> (f64, f64, f64) {
        (self.r, self.g, self.b)
    }

    pub fn of_hex(hex: &str) -> DomainResult<Self> {
        let raw = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
        let expanded = match raw.len() {
            3 => {
                let mut s = String::with_capacity(6);
                for ch in raw.chars() {
                    s.push(ch);
                    s.push(ch);
                }
                s
            }
            6 => raw.to_string(),
            _ => return Err(DomainError::InvalidHex(hex.to_string())),
        };
        let packed = u32::from_str_radix(&expanded, 16)
            .map_err(|_| DomainError::InvalidHex(hex.to_string()))?;
        Ok(Self::new(
            ((packed >> 16) & 0xFF) as f64 / 255.0,
            ((packed >> 8) & 0xFF) as f64 / 255.0,
            (packed & 0xFF) as f64 / 255.0,
        ))
    }

    pub fn to_hex(&self) -> String {
        let r = (self.r * 255.0).round() as u8;
        let g = (self.g * 255.0).round() as u8;
        let b = (self.b * 255.0).round() as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }

    /// Relative luminance (WCAG), useful for choosing a readable text color on a swatch.
    pub fn relative_luminance(&self) -> f64 {
        0.2126 * srgb_to_linear(self.r)
            + 0.7152 * srgb_to_linear(self.g)
            + 0.0722 * srgb_to_linear(self.b)
    }

    pub fn to_lab(&self) -> Lab {
        const XN: f64 = 0.95047;
        const YN: f64 = 1.00000;
        const ZN: f64 = 1.08883;

        let lr = srgb_to_linear(self.r);
        let lg = srgb_to_linear(self.g);
        let lb = srgb_to_linear(self.b);

        let x = (0.4124564 * lr + 0.3575761 * lg + 0.1804375 * lb) / XN;
        let y = (0.2126729 * lr + 0.7151522 * lg + 0.0721750 * lb) / YN;
        let z = (0.0193339 * lr + 0.1191920 * lg + 0.9503041 * lb) / ZN;

        let fx = pivot(x);
        let fy = pivot(y);
        let fz = pivot(z);

        Lab {
            l: 116.0 * fy - 16.0,
            a: 500.0 * (fx - fy),
            b: 200.0 * (fy - fz),
        }
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

/// Color in the CIE L*a*b* space (D65 illuminant, 2-degree observer).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Lab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

#[inline]
pub fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
pub fn linear_to_srgb(c: f64) -> f64 {
    let v = c.clamp(0.0, 1.0);
    if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

#[inline]
fn pivot(t: f64) -> f64 {
    if t > 216.0 / 24389.0 {
        t.cbrt()
    } else {
        (24389.0 / 27.0 * t + 16.0) / 116.0
    }
}

/// CIEDE2000 color difference between two `Rgb` colors.
#[inline]
pub fn delta_e_2000(first: Rgb, second: Rgb) -> f64 {
    delta_e_2000_lab(first.to_lab(), second.to_lab())
}

/// CIEDE2000 color difference between two `Lab` coordinates.
pub fn delta_e_2000_lab(lab1: Lab, lab2: Lab) -> f64 {
    let k_l = 1.0;
    let k_c = 1.0;
    let k_h = 1.0;

    let c1 = lab1.a.hypot(lab1.b);
    let c2 = lab2.a.hypot(lab2.b);
    let c_bar = (c1 + c2) / 2.0;

    let c_bar7 = c_bar.powi(7);
    let twenty_five7 = 25.0_f64.powi(7);
    let g = 0.5 * (1.0 - (c_bar7 / (c_bar7 + twenty_five7)).sqrt());

    let a1p = (1.0 + g) * lab1.a;
    let a2p = (1.0 + g) * lab2.a;
    let c1p = a1p.hypot(lab1.b);
    let c2p = a2p.hypot(lab2.b);

    let h1p = hue_angle(lab1.b, a1p);
    let h2p = hue_angle(lab2.b, a2p);

    let d_lp = lab2.l - lab1.l;
    let d_cp = c2p - c1p;

    let dhp = if c1p * c2p == 0.0 {
        0.0
    } else if (h2p - h1p).abs() <= 180.0 {
        h2p - h1p
    } else if h2p - h1p > 180.0 {
        h2p - h1p - 360.0
    } else {
        h2p - h1p + 360.0
    };
    let d_hp = 2.0 * (c1p * c2p).sqrt() * (dhp.to_radians() / 2.0).sin();

    let l_bar_p = (lab1.l + lab2.l) / 2.0;
    let c_bar_p = (c1p + c2p) / 2.0;

    let h_bar_p = if c1p * c2p == 0.0 {
        h1p + h2p
    } else if (h1p - h2p).abs() <= 180.0 {
        (h1p + h2p) / 2.0
    } else if h1p + h2p < 360.0 {
        (h1p + h2p + 360.0) / 2.0
    } else {
        (h1p + h2p - 360.0) / 2.0
    };

    let t = 1.0 - 0.17 * (h_bar_p - 30.0).to_radians().cos()
        + 0.24 * (2.0 * h_bar_p).to_radians().cos()
        + 0.32 * (3.0 * h_bar_p + 6.0).to_radians().cos()
        - 0.20 * (4.0 * h_bar_p - 63.0).to_radians().cos();

    let d_theta = 30.0 * (-((h_bar_p - 275.0) / 25.0).powi(2)).exp();
    let c_bar_p7 = c_bar_p.powi(7);
    let r_c = 2.0 * (c_bar_p7 / (c_bar_p7 + twenty_five7)).sqrt();
    let r_t = -r_c * (2.0 * d_theta.to_radians()).sin();

    let l_bar_p50 = (l_bar_p - 50.0).powi(2);
    let s_l = 1.0 + (0.015 * l_bar_p50) / (20.0 + l_bar_p50).sqrt();
    let s_c = 1.0 + 0.045 * c_bar_p;
    let s_h = 1.0 + 0.015 * c_bar_p * t;

    let term_l = d_lp / (k_l * s_l);
    let term_c = d_cp / (k_c * s_c);
    let term_h = d_hp / (k_h * s_h);

    (term_l * term_l + term_c * term_c + term_h * term_h + r_t * term_c * term_h).sqrt()
}

#[inline]
fn hue_angle(b: f64, ap: f64) -> f64 {
    if b == 0.0 && ap == 0.0 {
        return 0.0;
    }
    let deg = b.atan2(ap) * 180.0 / PI;
    if deg >= 0.0 { deg } else { deg + 360.0 }
}

/// Average of a set of pixels in linear light.
pub fn average(samples: &[Rgb]) -> DomainResult<Rgb> {
    if samples.is_empty() {
        return Err(DomainError::InvalidArgument("Échantillon vide".to_string()));
    }
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    for sample in samples {
        r += srgb_to_linear(sample.r);
        g += srgb_to_linear(sample.g);
        b += srgb_to_linear(sample.b);
    }
    let count = samples.len() as f64;
    Ok(Rgb::new(
        linear_to_srgb(r / count),
        linear_to_srgb(g / count),
        linear_to_srgb(b / count),
    ))
}

/// Corrects the color cast of a photo from a reference point assumed to be neutral grey.
pub fn neutralise(sample: Rgb, grey_point: Rgb) -> Rgb {
    let gr = srgb_to_linear(grey_point.r).max(1e-4);
    let gg = srgb_to_linear(grey_point.g).max(1e-4);
    let gb = srgb_to_linear(grey_point.b).max(1e-4);

    let target = (gr + gg + gb) / 3.0;

    Rgb::new(
        linear_to_srgb(srgb_to_linear(sample.r) * target / gr),
        linear_to_srgb(srgb_to_linear(sample.g) * target / gg),
        linear_to_srgb(srgb_to_linear(sample.b) * target / gb),
    )
}

/// Subtractive color mixing (single-constant Kubelka-Munk).
pub fn mix(colors: &[Rgb], weights: &[f64]) -> DomainResult<Rgb> {
    if colors.is_empty() || colors.len() != weights.len() {
        return Err(DomainError::InvalidArgument(
            "Il faut autant de poids que de couleurs, et au moins une couleur".to_string(),
        ));
    }
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return Err(DomainError::InvalidArgument(
            "La somme des poids doit être strictement positive".to_string(),
        ));
    }

    let mut ks = [0.0; 3];
    for (c, &weight) in colors.iter().zip(weights.iter()) {
        let w = weight / total;
        ks[0] += w * to_ks_channel(srgb_to_linear(c.r));
        ks[1] += w * to_ks_channel(srgb_to_linear(c.g));
        ks[2] += w * to_ks_channel(srgb_to_linear(c.b));
    }
    Ok(from_ks(ks))
}

/// Kubelka-Munk K/S coordinates of a color, one channel per component.
#[inline]
pub fn to_ks(color: Rgb) -> [f64; 3] {
    [
        to_ks_channel(srgb_to_linear(color.r)),
        to_ks_channel(srgb_to_linear(color.g)),
        to_ks_channel(srgb_to_linear(color.b)),
    ]
}

/// Color corresponding to Kubelka-Munk K/S coordinates.
#[inline]
pub fn from_ks(ks: [f64; 3]) -> Rgb {
    Rgb::new(
        linear_to_srgb(from_ks_channel(ks[0])),
        linear_to_srgb(from_ks_channel(ks[1])),
        linear_to_srgb(from_ks_channel(ks[2])),
    )
}

#[inline]
fn to_ks_channel(reflectance: f64) -> f64 {
    let r = reflectance.clamp(0.002, 0.998);
    (1.0 - r) * (1.0 - r) / (2.0 * r)
}

#[inline]
fn from_ks_channel(ks: f64) -> f64 {
    1.0 + ks - (ks * ks + 2.0 * ks).sqrt()
}

/// A colorant described by its Kubelka-Munk constants: absorption `K` and scattering `S` per channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colorant {
    k: [f64; 3],
    s: [f64; 3],
}

impl Colorant {
    pub const REFERENCE_TINT_CONCENTRATION: f64 = 0.1;
    const REFERENCE_SCATTERING: f64 = 1.0;
    const MIN_SCATTERING: f64 = 1e-4;

    #[inline]
    pub fn absorption(&self) -> [f64; 3] {
        self.k
    }

    #[inline]
    pub fn scattering(&self) -> [f64; 3] {
        self.s
    }

    /// Pure colorant color straight out of the tube (masstone).
    pub fn masstone(&self) -> Rgb {
        from_ks([
            self.k[0] / self.s[0],
            self.k[1] / self.s[1],
            self.k[2] / self.s[2],
        ])
    }

    /// Colorant let down with reference white at the given concentration.
    pub fn tint(&self, concentration: f64) -> Rgb {
        Self::mix(
            &[*self, Self::of_masstone(Rgb::REFERENCE_WHITE)],
            &[concentration, 1.0 - concentration],
        )
        .unwrap_or(self.masstone())
    }

    /// Colorant described by its masstone alone (`S = 1.0` across all three channels).
    pub fn of_masstone(masstone: Rgb) -> Self {
        let ratio = to_ks(masstone);
        let mut k = [0.0; 3];
        let mut s = [0.0; 3];
        for channel in 0..3 {
            s[channel] = Self::REFERENCE_SCATTERING;
            k[channel] = ratio[channel] * Self::REFERENCE_SCATTERING;
        }
        Self { k, s }
    }

    /// Colorant fitted to two observations: masstone and diluted tint.
    pub fn of_masstone_and_tint(masstone: Rgb, tint: Rgb, concentration: f64) -> Self {
        let c = concentration.clamp(1e-3, 1.0 - 1e-3);
        let masstone_ratio = to_ks(masstone);
        let tint_ratio = to_ks(tint);
        let white_ratio = to_ks(Rgb::REFERENCE_WHITE);

        let mut k = [0.0; 3];
        let mut s = [0.0; 3];
        for channel in 0..3 {
            let white_absorption = white_ratio[channel] * Self::REFERENCE_SCATTERING;
            let denominator = c * (masstone_ratio[channel] - tint_ratio[channel]);
            let numerator =
                (1.0 - c) * (tint_ratio[channel] * Self::REFERENCE_SCATTERING - white_absorption);

            s[channel] = if denominator <= 0.0 || numerator <= 0.0 {
                Self::REFERENCE_SCATTERING
            } else {
                (numerator / denominator).max(Self::MIN_SCATTERING)
            };
            k[channel] = masstone_ratio[channel] * s[channel];
        }
        Self { k, s }
    }

    /// Mixes multiple colorants.
    pub fn mix(colorants: &[Colorant], weights: &[f64]) -> DomainResult<Rgb> {
        if colorants.is_empty() || colorants.len() != weights.len() {
            return Err(DomainError::InvalidArgument(
                "Il faut autant de poids que de colorants, et au moins un colorant".to_string(),
            ));
        }
        let total: f64 = weights.iter().sum();
        if total <= 0.0 {
            return Err(DomainError::InvalidArgument(
                "La somme des poids doit être strictement positive".to_string(),
            ));
        }

        let mut k = [0.0; 3];
        let mut s = [0.0; 3];
        for (colorant, &weight) in colorants.iter().zip(weights.iter()) {
            let share = weight / total;
            for channel in 0..3 {
                k[channel] += share * colorant.k[channel];
                s[channel] += share * colorant.s[channel];
            }
        }
        Ok(Self::color_of(k, s))
    }

    /// Color of a mixture whose absorption and scattering are already accumulated.
    #[inline]
    pub fn color_of(absorption: [f64; 3], scattering: [f64; 3]) -> Rgb {
        from_ks([
            absorption[0] / scattering[0].max(Self::MIN_SCATTERING),
            absorption[1] / scattering[1].max(Self::MIN_SCATTERING),
            absorption[2] / scattering[2].max(Self::MIN_SCATTERING),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip_preserves_color() {
        assert_eq!("#4A3427", Rgb::of_hex("#4A3427").unwrap().to_hex());
        assert_eq!("#4A3427", Rgb::of_hex("4a3427").unwrap().to_hex());
        assert_eq!("#FFFFFF", Rgb::of_hex("#FFF").unwrap().to_hex());
    }

    #[test]
    fn color_has_zero_delta_with_itself() {
        let color = Rgb::of_hex("#7A3A22").unwrap();
        assert_eq!(0.0, delta_e_2000(color, color));
    }

    #[test]
    fn black_and_white_are_opposite() {
        let black = Rgb::of_hex("#000000").unwrap();
        let white = Rgb::of_hex("#FFFFFF").unwrap();
        assert!(delta_e_2000(black, white) > 90.0);
    }

    #[test]
    fn blue_and_yellow_make_green_not_grey() {
        let blue = Rgb::of_hex("#1F2E7A").unwrap();
        let yellow = Rgb::of_hex("#F6C500").unwrap();
        let mixed = mix(&[blue, yellow], &[1.0, 1.0]).unwrap();
        assert!(mixed.g() > mixed.r(), "verte doit dominer dans {}", mixed);
        assert!(mixed.g() > mixed.b(), "verte doit dominer dans {}", mixed);
    }

    #[test]
    fn mixing_color_with_itself_is_unchanged() {
        let umber = Rgb::of_hex("#4A3427").unwrap();
        let mixed = mix(&[umber, umber], &[3.0, 1.0]).unwrap();
        assert!(delta_e_2000(umber, mixed) < 1.0);
    }

    #[test]
    fn overwhelming_weight_dominates_mix() {
        let white = Rgb::of_hex("#F7F5F0").unwrap();
        let black = Rgb::of_hex("#1C1B1A").unwrap();
        let mostly_white = mix(&[white, black], &[99.0, 1.0]).unwrap();
        let half_and_half = mix(&[white, black], &[1.0, 1.0]).unwrap();
        assert!(mostly_white.relative_luminance() > half_and_half.relative_luminance());
        assert!(half_and_half.relative_luminance() < white.relative_luminance());
    }

    #[test]
    fn empty_or_zero_weight_mix_is_rejected() {
        assert!(mix(&[], &[]).is_err());
        assert!(mix(&[Rgb::of_hex("#000000").unwrap()], &[0.0]).is_err());
    }

    #[test]
    fn average_is_computed_in_linear_light() {
        let mid = average(&[
            Rgb::of_hex("#000000").unwrap(),
            Rgb::of_hex("#FFFFFF").unwrap(),
        ])
        .unwrap();
        assert_eq!("#BCBCBC", mid.to_hex());
    }

    #[test]
    fn averaging_same_color_is_unchanged() {
        let flesh = Rgb::of_hex("#C98F72").unwrap();
        assert_eq!(
            flesh.to_hex(),
            average(&[flesh, flesh, flesh]).unwrap().to_hex()
        );
    }

    #[test]
    fn empty_sample_average_is_rejected() {
        assert!(average(&[]).is_err());
    }

    #[test]
    fn neutralise_makes_reference_grey_neutral() {
        let warm_grey = Rgb::of_hex("#A08860").unwrap();
        let corrected = neutralise(warm_grey, warm_grey);
        assert!((corrected.r() - corrected.g()).abs() < 0.01);
        assert!((corrected.g() - corrected.b()).abs() < 0.01);
    }

    #[test]
    fn neutralise_removes_warm_lamp_cast() {
        let photographed_flesh = Rgb::of_hex("#D9A070").unwrap();
        let photographed_white = Rgb::of_hex("#E8D2A8").unwrap();
        let corrected = neutralise(photographed_flesh, photographed_white);
        assert!(
            (corrected.r() - corrected.b()) < (photographed_flesh.r() - photographed_flesh.b())
        );
    }

    #[test]
    fn neutral_reference_point_changes_nothing() {
        let flesh = Rgb::of_hex("#C98F72").unwrap();
        let corrected = neutralise(flesh, Rgb::of_hex("#808080").unwrap());
        assert!(delta_e_2000(flesh, corrected) < 0.5);
    }

    #[test]
    fn out_of_bounds_components_are_clamped() {
        assert_eq!("#FFFFFF", Rgb::new(1.0000000000000002, 1.5, 42.0).to_hex());
        assert_eq!("#000000", Rgb::new(-1e-15, -0.5, -3.0).to_hex());
    }

    #[test]
    fn ivory_black_two_constant_divergence() {
        // Ivory black is warm out of the tube (#221F1C) and cool in a diluted tint (#8E9094)
        let masstone = Rgb::of_hex("#221F1C").unwrap();
        let tint = Rgb::of_hex("#8E9094").unwrap();
        let colorant = Colorant::of_masstone_and_tint(
            masstone,
            tint,
            Colorant::REFERENCE_TINT_CONCENTRATION,
        );
        assert_eq!(masstone.to_hex(), colorant.masstone().to_hex());
        let reproduced_tint = colorant.tint(Colorant::REFERENCE_TINT_CONCENTRATION);
        assert!(delta_e_2000(tint, reproduced_tint) < 0.5);
        // In a diluted tint, blue exceeds red (cool grey)
        assert!(reproduced_tint.b() > reproduced_tint.r());
    }
}
