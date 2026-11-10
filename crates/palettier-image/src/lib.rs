//! Safe image decoding, linear-light sampling, and dominant color extraction.

use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, ImageReader, Limits, Rgb as ImgRgb, RgbImage};
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::image::{ImagePalette, PixelMap};
use palettier_domain::project::DominantColour;
use std::io::Cursor;

/// Safety limits for image decoding (anti-Decompression Bomb / OOM).
const MAX_IMAGE_DIMENSION: u32 = 8192;
const MAX_ALLOC_BYTES: u64 = 256 * 1024 * 1024; // 256 MB max

#[derive(Debug, Clone, Default)]
pub struct ImageDecoder;

impl ImageDecoder {
    pub fn new() -> Self {
        Self
    }

    fn read_bounded(bytes: &[u8]) -> DomainResult<DynamicImage> {
        let mut reader = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| DomainError::InvalidArgument(format!("Format d'image non reconnu : {e}")))?;

        if reader.format().is_none() {
            return Err(DomainError::InvalidArgument(
                "Format d'image non reconnu".to_string(),
            ));
        }

        let mut limits = Limits::default();
        limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
        limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
        limits.max_alloc = Some(MAX_ALLOC_BYTES);
        reader.limits(limits);

        reader
            .decode()
            .map_err(|e| DomainError::InvalidArgument(format!("Format d'image non reconnu : {e}")))
    }

    /// Decodes an image into a `PixelMap` (`0xAARRGGBB`).
    pub fn decode(&self, bytes: &[u8]) -> DomainResult<PixelMap> {
        let img = Self::read_bounded(bytes)?;
        let rgba = img.to_rgba8();
        let width = rgba.width() as usize;
        let height = rgba.height() as usize;
        let mut argb = Vec::with_capacity(width * height);
        for pixel in rgba.pixels() {
            let [r, g, b, a] = pixel.0;
            let packed =
                ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            argb.push(packed);
        }
        PixelMap::new(width, height, argb)
    }

    /// Downscales an image to its maximum edge (`max_edge`) and re-encodes it as JPEG (quality 85).
    /// If the image is already smaller than or equal to `max_edge`, returns the original bytes unchanged.
    pub fn scale_to(&self, bytes: &[u8], max_edge: u32) -> DomainResult<Vec<u8>> {
        let source = Self::read_bounded(bytes)?;
        let max_dim = source.width().max(source.height());
        if max_dim <= max_edge {
            return Ok(bytes.to_vec());
        }

        let scale = (max_edge as f64) / (max_dim as f64);
        let width = ((source.width() as f64 * scale).round() as u32).max(1);
        let height = ((source.height() as f64 * scale).round() as u32).max(1);

        let resized = source.resize_exact(width, height, FilterType::Triangle);
        let rgb: RgbImage = resized.to_rgb8();

        let mut out = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut out, 85);
        encoder
            .encode_image(&rgb)
            .map_err(|e| DomainError::InvalidArgument(format!("Échec d'encodage JPEG : {e}")))?;
        Ok(out)
    }

    /// Extracts the `count` dominant colors directly from image bytes.
    pub fn dominant_colours(&self, bytes: &[u8], count: usize) -> DomainResult<Vec<DominantColour>> {
        let map = self.decode(bytes)?;
        Ok(ImagePalette::dominant(&map, count))
    }

    /// Encodes a `PixelMap` as PNG (useful for displaying value studies and Notan).
    pub fn encode_png(&self, map: &PixelMap) -> DomainResult<Vec<u8>> {
        let mut img = RgbImage::new(map.width as u32, map.height as u32);
        for y in 0..map.height {
            for x in 0..map.width {
                let packed = map.argb[y * map.width + x];
                let r = ((packed >> 16) & 0xFF) as u8;
                let g = ((packed >> 8) & 0xFF) as u8;
                let b = (packed & 0xFF) as u8;
                img.put_pixel(x as u32, y as u32, ImgRgb([r, g, b]));
            }
        }
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .map_err(|e| DomainError::InvalidArgument(format!("Échec d'encodage PNG : {e}")))?;
        Ok(buf)
    }

    /// Generates a high-quality 900x900 square Social Recipe Card PNG.
    pub fn render_social_recipe_card(
        &self,
        photo_bytes: Option<&[u8]>,
        swatches_hex: &[String],
    ) -> DomainResult<Vec<u8>> {
        let width = 900u32;
        let height = 900u32;
        let mut card = RgbImage::from_pixel(width, height, ImgRgb([20, 22, 28]));

        // 1. Gold studio accent border at the top
        for x in 0..width {
            card.put_pixel(x, 0, ImgRgb([200, 155, 110]));
            card.put_pixel(x, 1, ImgRgb([200, 155, 110]));
            card.put_pixel(x, 2, ImgRgb([200, 155, 110]));
        }

        // 2. If photo is present, resize and composite into main canvas
        if let Some(bytes) = photo_bytes
            && let Ok(dyn_img) = Self::read_bounded(bytes)
        {
            let target_w = 780u32;
            let target_h = 580u32;
            let scaled = dyn_img.resize_to_fill(target_w, target_h, FilterType::Lanczos3);
            let rgb_scaled = scaled.to_rgb8();
            let start_x = (width - target_w) / 2;
            let start_y = 60u32;

            for y in 0..target_h {
                for x in 0..target_w {
                    card.put_pixel(start_x + x, start_y + y, *rgb_scaled.get_pixel(x, y));
                }
            }
        }

        // 3. Draw color swatch ladder at the bottom
        let swatch_count = swatches_hex.len().max(1) as u32;
        let margin_x = 60u32;
        let total_w = width - (margin_x * 2);
        let swatch_w = (total_w / swatch_count).saturating_sub(12).max(40);
        let swatch_h = 110u32;
        let start_y = 680u32;

        for (idx, hex) in swatches_hex.iter().enumerate() {
            let col = palettier_domain::color::Rgb::of_hex(hex)
                .unwrap_or_else(|_| palettier_domain::color::Rgb::new(0.5, 0.5, 0.5));
            let r = (col.r() * 255.0).round() as u8;
            let g = (col.g() * 255.0).round() as u8;
            let b = (col.b() * 255.0).round() as u8;

            let cur_x = margin_x + (idx as u32) * (swatch_w + 12);
            if cur_x + swatch_w <= width - margin_x {
                for sy in 0..swatch_h {
                    for sx in 0..swatch_w {
                        card.put_pixel(cur_x + sx, start_y + sy, ImgRgb([r, g, b]));
                    }
                }
            }
        }

        // 4. Encode card as PNG
        let mut buf = Vec::new();
        card.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .map_err(|e| DomainError::InvalidArgument(format!("Failed to encode social card PNG: {e}")))?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, Rgb as ImgRgb};
    use palettier_domain::color::Rgb;

    fn two_tone_png(left_hex: &str, right_hex: &str, left_share: f64) -> Vec<u8> {
        let left = Rgb::of_hex(left_hex).unwrap();
        let right = Rgb::of_hex(right_hex).unwrap();
        let left_px = ImgRgb([
            (left.r() * 255.0).round() as u8,
            (left.g() * 255.0).round() as u8,
            (left.b() * 255.0).round() as u8,
        ]);
        let right_px = ImgRgb([
            (right.r() * 255.0).round() as u8,
            (right.g() * 255.0).round() as u8,
            (right.b() * 255.0).round() as u8,
        ]);

        let split = (200.0 * left_share) as u32;
        let mut img = RgbImage::new(200, 100);
        for y in 0..100 {
            for x in 0..200 {
                img.put_pixel(x, y, if x < split { left_px } else { right_px });
            }
        }
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .unwrap();
        buf
    }

    #[test]
    fn image_decodes_to_pixels_with_dimensions() {
        let decoder = ImageDecoder::new();
        let pixels = decoder
            .decode(&two_tone_png("#C98F72", "#8A5F4A", 0.5))
            .unwrap();
        assert_eq!(200, pixels.width);
        assert_eq!(100, pixels.height);
        assert_eq!(200 * 100, pixels.argb.len());
        assert!(!pixels.is_empty());
    }

    #[test]
    fn two_tones_emerge_with_their_proportions() {
        let decoder = ImageDecoder::new();
        let dominant = ImagePalette::dominant(
            &decoder
                .decode(&two_tone_png("#C98F72", "#8A5F4A", 0.75))
                .unwrap(),
            2,
        );
        assert_eq!(2, dominant.len());
        assert_eq!("#C98F72", dominant[0].color.to_hex());
        assert!((dominant[0].share - 0.75).abs() < 0.05);
        assert_eq!("#8A5F4A", dominant[1].color.to_hex());
    }

    #[test]
    fn dominant_extraction_is_deterministic() {
        let decoder = ImageDecoder::new();
        let raw = two_tone_png("#C98F72", "#8A5F4A", 0.5);
        let once = ImagePalette::dominant(&decoder.decode(&raw).unwrap(), 3);
        let twice = ImagePalette::dominant(&decoder.decode(&raw).unwrap(), 3);
        assert_eq!(once, twice);
    }

    #[test]
    fn large_image_is_reduced_and_small_is_kept_as_is() {
        let decoder = ImageDecoder::new();
        let mut big_img = RgbImage::new(2400, 1800);
        for p in big_img.pixels_mut() {
            *p = ImgRgb([220, 140, 80]);
        }
        let mut big_bytes = Vec::new();
        big_img
            .write_to(&mut Cursor::new(&mut big_bytes), ImageFormat::Png)
            .unwrap();

        let scaled = decoder.scale_to(&big_bytes, 1024).unwrap();
        let reduced = decoder.decode(&scaled).unwrap();
        assert_eq!(1024, reduced.width);
        assert_eq!(768, reduced.height);

        let small = two_tone_png("#C98F72", "#8A5F4A", 0.5);
        assert_eq!(small, decoder.scale_to(&small, 1024).unwrap());
    }

    #[test]
    fn invalid_image_bytes_are_cleanly_rejected() {
        let decoder = ImageDecoder::new();
        assert!(decoder.decode(b"pas une image").is_err());
    }
}
