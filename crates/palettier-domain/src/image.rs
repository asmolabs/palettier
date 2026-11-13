use crate::color::{Lab, Rgb, average, delta_e_2000_lab, linear_to_srgb, srgb_to_linear};
use crate::error::{DomainError, DomainResult};
use crate::i18n::Locale;
use crate::project::DominantColour;

/// Visual study mode for reading a reference photo at the workbench.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudyMode {
    /// Original color.
    Color,
    /// Grisaille quantized onto the 9-step studio value scale.
    Grisaille9,
    /// 5-value academic scale (Deep shadow, Shadow, Halftone, Light, Highlight).
    Academy5,
    /// 3-mass Notan (Shadow, Midtone, Light).
    Notan3,
    /// Terminator line (exact border between light and shadow).
    Terminator,
    /// Zenithal priming (simulates 45° top-down spray undercoat and ambient occlusion).
    Zenithal,
    /// "Squint": mass blur that simplifies fine details.
    Squint,
}

impl StudyMode {
    pub const ALL: [StudyMode; 7] = [
        StudyMode::Color,
        StudyMode::Grisaille9,
        StudyMode::Academy5,
        StudyMode::Notan3,
        StudyMode::Terminator,
        StudyMode::Zenithal,
        StudyMode::Squint,
    ];

    pub fn label(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (StudyMode::Color, Locale::Fr) => "Couleur réelle",
            (StudyMode::Grisaille9, Locale::Fr) => "Grisaille 9 valeurs",
            (StudyMode::Academy5, Locale::Fr) => "Académie 5 valeurs",
            (StudyMode::Notan3, Locale::Fr) => "Notan 3 masses",
            (StudyMode::Terminator, Locale::Fr) => "Ligne de terminateur",
            (StudyMode::Zenithal, Locale::Fr) => "Sous-couche zénithale",
            (StudyMode::Squint, Locale::Fr) => "Plisser les yeux",
            (StudyMode::Color, Locale::En) => "True color",
            (StudyMode::Grisaille9, Locale::En) => "9-value Grisaille",
            (StudyMode::Academy5, Locale::En) => "5-value Academic",
            (StudyMode::Notan3, Locale::En) => "3-mass Notan",
            (StudyMode::Terminator, Locale::En) => "Terminator line",
            (StudyMode::Zenithal, Locale::En) => "Zenithal priming",
            (StudyMode::Squint, Locale::En) => "Squint",
        }
    }
}

/// An image reduced to what the domain needs: packed ARGB pixels (`0xAARRGGBB`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelMap {
    pub width: usize,
    pub height: usize,
    pub argb: Vec<u32>,
}

impl PixelMap {
    pub fn new(width: usize, height: usize, argb: Vec<u32>) -> DomainResult<Self> {
        if argb.len() != width * height {
            return Err(DomainError::InvalidArgument(format!(
                "{} pixels pour {}x{}",
                argb.len(),
                width,
                height
            )));
        }
        Ok(Self {
            width,
            height,
            argb,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.argb.is_empty()
    }

    /// Samples the average color (in linear light) around a point `(x, y)` in the image.
    pub fn sample_around(&self, x: isize, y: isize, radius: isize) -> Option<Rgb> {
        if self.is_empty()
            || x < 0
            || y < 0
            || x >= self.width as isize
            || y >= self.height as isize
        {
            return None;
        }

        let r = radius.max(0);
        let row_start = (y - r).max(0) as usize;
        let row_end = (y + r).min(self.height as isize - 1) as usize;
        let col_start = (x - r).max(0) as usize;
        let col_end = (x + r).min(self.width as isize - 1) as usize;

        let mut samples = Vec::new();
        for row in row_start..=row_end {
            for col in col_start..=col_end {
                let packed = self.argb[row * self.width + col];
                samples.push(argb_to_rgb(packed));
            }
        }
        average(&samples).ok()
    }

    /// Transforms the image according to a value-study mode (9-value Grisaille, 3-mass Notan, Squint).
    pub fn to_study_map(&self, mode: StudyMode) -> Self {
        match mode {
            StudyMode::Color => self.clone(),
            StudyMode::Grisaille9 => {
                // 9 lightness steps L* (from 8 to 94)
                let argb = self
                    .argb
                    .iter()
                    .map(|&packed| {
                        let rgb = argb_to_rgb(packed);
                        let l_star = rgb.to_lab().l.clamp(0.0, 100.0);
                        let step = ((l_star / 100.0) * 8.99).floor() as usize;
                        let quant_l = 8.0 + (step as f64) * (86.0 / 8.0);
                        // Convert neutral L* (a*=0, b*=0) to a warm studio grey sRGB
                        let v = ((quant_l / 100.0).powf(1.1) * 255.0).round().clamp(0.0, 255.0) as u32;
                        let r = (v + 3).min(255);
                        let g = v;
                        let b = v.saturating_sub(3);
                        0xFF00_0000 | (r << 16) | (g << 8) | b
                    })
                    .collect();
                Self {
                    width: self.width,
                    height: self.height,
                    argb,
                }
            }
            StudyMode::Notan3 => {
                let argb = self
                    .argb
                    .iter()
                    .map(|&packed| {
                        let rgb = argb_to_rgb(packed);
                        let l_star = rgb.to_lab().l;
                        if l_star < 36.0 {
                            0xFF21_1D1A // Shadow mass
                        } else if l_star < 68.0 {
                            0xFF85_7E75 // Midtone mass
                        } else {
                            0xFFED_E7DD // Light mass
                        }
                    })
                    .collect();
                Self {
                    width: self.width,
                    height: self.height,
                    argb,
                }
            }
            StudyMode::Academy5 => {
                let argb = self
                    .argb
                    .iter()
                    .map(|&packed| {
                        let rgb = argb_to_rgb(packed);
                        let l = rgb.to_lab().l;
                        if l < 20.0 {
                            0xFF12_100E // Deep occlusion
                        } else if l < 40.0 {
                            0xFF3B_3530 // Form shadow
                        } else if l < 60.0 {
                            0xFF75_6C64 // Halftone
                        } else if l < 80.0 {
                            0xFFBA_B1A5 // Key light
                        } else {
                            0xFFF5_F0E6 // Catchlight highlight
                        }
                    })
                    .collect();
                Self {
                    width: self.width,
                    height: self.height,
                    argb,
                }
            }
            StudyMode::Terminator => {
                let argb = self
                    .argb
                    .iter()
                    .map(|&packed| {
                        let rgb = argb_to_rgb(packed);
                        let l = rgb.to_lab().l;
                        if (46.0..=52.0).contains(&l) {
                            0xFF00_E5FF // High-visibility Cyan terminator boundary
                        } else {
                            let g = (l * 2.55).round() as u32;
                            0xFF00_0000 | (g << 16) | (g << 8) | g
                        }
                    })
                    .collect();
                Self {
                    width: self.width,
                    height: self.height,
                    argb,
                }
            }
            StudyMode::Zenithal => {
                let argb = self
                    .argb
                    .iter()
                    .map(|&packed| {
                        let rgb = argb_to_rgb(packed);
                        let l = rgb.to_lab().l;
                        let z = if l < 25.0 {
                            (l * 0.4).round() as u32
                        } else if l < 70.0 {
                            (10.0 + (l - 25.0) * 2.8).min(210.0).round() as u32
                        } else {
                            (210.0 + (l - 70.0) * 1.5).min(255.0).round() as u32
                        };
                        0xFF00_0000 | (z << 16) | (z << 8) | z
                    })
                    .collect();
                Self {
                    width: self.width,
                    height: self.height,
                    argb,
                }
            }
            StudyMode::Squint => {
                if self.is_empty() {
                    return self.clone();
                }
                let radius = ((self.width.max(self.height) / 70) as isize).clamp(3, 12);
                let step = (radius as usize / 2).max(1);
                let mut out = vec![0u32; self.argb.len()];
                let mut y = 0;
                while y < self.height {
                    let mut x = 0;
                    while x < self.width {
                        let c = self
                            .sample_around(x as isize, y as isize, radius)
                            .unwrap_or_else(|| Rgb::new(0.5, 0.5, 0.5));
                        let r = (c.r() * 255.0).round() as u32;
                        let g = (c.g() * 255.0).round() as u32;
                        let b = (c.b() * 255.0).round() as u32;
                        let packed = 0xFF00_0000 | (r << 16) | (g << 8) | b;
                        for dy in 0..step {
                            for dx in 0..step {
                                if y + dy < self.height && x + dx < self.width {
                                    out[(y + dy) * self.width + (x + dx)] = packed;
                                }
                            }
                        }
                        x += step;
                    }
                    y += step;
                }
                Self {
                    width: self.width,
                    height: self.height,
                    argb: out,
                }
            }
        }
    }
}

#[inline]
fn argb_to_rgb(packed: u32) -> Rgb {
    Rgb::new(
        ((packed >> 16) & 0xFF) as f64 / 255.0,
        ((packed >> 8) & 0xFF) as f64 / 255.0,
        (packed & 0xFF) as f64 / 255.0,
    )
}

/// Dominant colors extracted from a photo (deterministic K-means in L*a*b*, averaged in linear light).
pub struct ImagePalette;

impl ImagePalette {
    const ANALYSIS_EDGE: usize = 160;
    const ITERATIONS: usize = 12;

    pub fn dominant(image: &PixelMap, count: usize) -> Vec<DominantColour> {
        let pixels = Self::linear_pixels(image);
        if pixels.is_empty() || count < 1 {
            return Vec::new();
        }

        let groups = count.min(pixels.len());
        let mut centres = Self::seed(&pixels, groups);
        let mut assignment = vec![0usize; pixels.len()];

        for _ in 0..Self::ITERATIONS {
            let centre_lab: Vec<Lab> = centres.iter().map(|c| Self::to_lab_point(*c)).collect();
            let mut moved = false;

            for (i, pixel) in pixels.iter().enumerate() {
                let best = Self::nearest(*pixel, &centre_lab);
                if assignment[i] != best {
                    assignment[i] = best;
                    moved = true;
                }
            }
            Self::recentre(&pixels, &assignment, &mut centres);
            if !moved {
                break;
            }
        }

        Self::summarise(&pixels, &assignment, &centres)
    }

    fn linear_pixels(image: &PixelMap) -> Vec<[f64; 3]> {
        if image.is_empty() {
            return Vec::new();
        }
        let step = (image.width.max(image.height) / Self::ANALYSIS_EDGE).max(1);
        let mut pixels = Vec::new();
        let mut y = 0;
        while y < image.height {
            let mut x = 0;
            while x < image.width {
                let packed = image.argb[y * image.width + x];
                pixels.push([
                    srgb_to_linear(((packed >> 16) & 0xFF) as f64 / 255.0),
                    srgb_to_linear(((packed >> 8) & 0xFF) as f64 / 255.0),
                    srgb_to_linear((packed & 0xFF) as f64 / 255.0),
                ]);
                x += step;
            }
            y += step;
        }
        pixels
    }

    fn seed(pixels: &[[f64; 3]], groups: usize) -> Vec<[f64; 3]> {
        let mut centres = Vec::with_capacity(groups);
        centres.push(pixels[0]);

        while centres.len() < groups {
            let chosen: Vec<Lab> = centres.iter().map(|c| Self::to_lab_point(*c)).collect();
            let mut farthest = 0;
            let mut worst = -1.0;
            for (i, p) in pixels.iter().enumerate() {
                let lab = Self::to_lab_point(*p);
                let nearest_dist = chosen
                    .iter()
                    .map(|c| delta_e_2000_lab(lab, *c))
                    .reduce(f64::min)
                    .unwrap_or(0.0);
                if nearest_dist > worst {
                    worst = nearest_dist;
                    farthest = i;
                }
            }
            centres.push(pixels[farthest]);
        }
        centres
    }

    fn nearest(pixel: [f64; 3], centres: &[Lab]) -> usize {
        let lab = Self::to_lab_point(pixel);
        let mut best = 0;
        let mut shortest = f64::MAX;
        for (i, c) in centres.iter().enumerate() {
            let distance = delta_e_2000_lab(lab, *c);
            if distance < shortest {
                shortest = distance;
                best = i;
            }
        }
        best
    }

    fn recentre(pixels: &[[f64; 3]], assignment: &[usize], centres: &mut [[f64; 3]]) {
        let mut sums = vec![[0.0; 3]; centres.len()];
        let mut counts = vec![0usize; centres.len()];

        for (i, &group) in assignment.iter().enumerate() {
            counts[group] += 1;
            for channel in 0..3 {
                sums[group][channel] += pixels[i][channel];
            }
        }
        for (group, centre) in centres.iter_mut().enumerate() {
            if counts[group] > 0 {
                let cnt = counts[group] as f64;
                for channel in 0..3 {
                    centre[channel] = sums[group][channel] / cnt;
                }
            }
        }
    }

    fn summarise(
        pixels: &[[f64; 3]],
        assignment: &[usize],
        centres: &[[f64; 3]],
    ) -> Vec<DominantColour> {
        let mut counts = vec![0usize; centres.len()];
        for &group in assignment {
            counts[group] += 1;
        }

        let total = pixels.len() as f64;
        let mut out: Vec<DominantColour> = centres
            .iter()
            .enumerate()
            .filter(|(i, _)| counts[*i] > 0)
            .map(|(i, c)| DominantColour {
                color: Self::to_rgb(*c),
                share: counts[i] as f64 / total,
            })
            .collect();
        out.sort_by(|a, b| b.share.total_cmp(&a.share));
        out
    }

    #[inline]
    fn to_rgb(lin: [f64; 3]) -> Rgb {
        Rgb::new(
            linear_to_srgb(lin[0]),
            linear_to_srgb(lin[1]),
            linear_to_srgb(lin[2]),
        )
    }

    #[inline]
    fn to_lab_point(lin: [f64; 3]) -> Lab {
        Self::to_rgb(lin).to_lab()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_around_averages_in_linear_light_and_clamps_bounds() {
        // 2x1: black on the left, white on the right -> linear average = #BCBCBC
        let map = PixelMap::new(2, 1, vec![0xFF000000, 0xFFFFFFFF]).unwrap();
        assert_eq!("#000000", map.sample_around(0, 0, 0).unwrap().to_hex());
        assert_eq!("#FFFFFF", map.sample_around(1, 0, 0).unwrap().to_hex());
        assert_eq!("#BCBCBC", map.sample_around(0, 0, 1).unwrap().to_hex());
        assert!(map.sample_around(-1, 0, 1).is_none());
        assert!(map.sample_around(2, 0, 1).is_none());
    }
}
