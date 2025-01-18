//!
//!  # Contrast
//!
//! This module provides functions to calculate the contrast between colors and determine if
//! the contrast fulfills various WCAG requirements
//!

use std::cmp::Ordering;

pub type RGBA = (u8, u8, u8);

pub fn brightness(color: RGBA) -> f32 {
    let (r, g, b) = color;
    // https://stackoverflow.com/questions/596216/formula-to-determine-perceived-brightness-of-rgb-color
    //((r as f32).powf(2.) * 0.299 + (g as f32).powf(2.) * 0.587 + (b as f32).powf(2.) * 0.114).sqrt()
    // 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32
    let hsl = hsl::HSL::from_rgb(&[r, g, b]);
    hsl.l as f32
}

/// Calculate the WCAG contrast ratio between the two colors
///
/// https://www.w3.org/TR/WCAG/#dfn-contrast-ratio
pub fn contrast(color_a: RGBA, color_b: RGBA) -> f32 {
    let mut brightness_a = brightness(color_a) + 0.05;
    let mut brightness_b = brightness(color_b) + 0.05;

    if brightness_a < brightness_b {
        std::mem::swap(&mut brightness_a, &mut brightness_b);
    }

    brightness_a / brightness_b
}

/// Determine if a contrast fulfills the requirement for normal text
/// https://www.w3.org/TR/WCAG/#contrast-minimum
pub fn is_minimum_text_contrast(text_color: RGBA, background_color: RGBA) -> bool {
    contrast(text_color, background_color) >= 4.5
}

/// Determine if a contrast fulfills the enhanced requirement for normal text
/// https://www.w3.org/TR/WCAG/#contrast-enhanced
pub fn is_enhanced_text_contrast(text_color: RGBA, background_color: RGBA) -> bool {
    contrast(text_color, background_color) >= 7.0
}

/// Determine if a contrast fulfills the requirement for large text
/// https://www.w3.org/TR/WCAG/#contrast-minimum
pub fn is_minimum_large_text_contrast(text_color: RGBA, background_color: RGBA) -> bool {
    contrast(text_color, background_color) >= 3.0
}

/// Determine if a contrast fulfills the enhanced requirement for large text
/// https://www.w3.org/TR/WCAG/#contrast-enhanced
pub fn is_enhanced_large_text_contrast(text_color: RGBA, background_color: RGBA) -> bool {
    contrast(text_color, background_color) >= 4.5
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwatchColorContrast {
    pub swatch_a_idx: usize,
    pub swatch_b_idx: usize,
    pub color_a: RGBA,
    pub color_b: RGBA,
    pub contrast: f32,
}

/// Return the contrast between all colors in both swatches
pub fn swatch_color_contrast(
    swatch_a: impl IntoIterator<Item = RGBA>,
    swatch_b: impl IntoIterator<Item = RGBA> + Clone,
) -> Vec<SwatchColorContrast> {
    let mut out: Vec<_> = swatch_a
        .into_iter()
        .enumerate()
        .flat_map(|(swatch_a_idx, color_a)| {
            swatch_b
                .clone()
                .into_iter()
                .enumerate()
                .map(move |(swatch_b_idx, color_b)| SwatchColorContrast {
                    swatch_a_idx,
                    swatch_b_idx,
                    color_a,
                    color_b,
                    contrast: contrast(color_a, color_b),
                })
        })
        .collect();

    out.sort_by(|lhs, rhs| {
        if lhs.contrast < rhs.contrast {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    out
}

#[cfg(test)]
mod test {
    use crate::contrast::SwatchColorContrast;

    #[test]
    fn test_swatch_contrasts() {
        let swatch_a = (0..20).map(|c| (c, c, c)).collect::<Vec<_>>();

        let swatch_b = (0..20).map(|c| (c, c, c)).collect::<Vec<_>>();

        let contrasts = super::swatch_color_contrast(swatch_a.clone(), swatch_b.clone());

        let text_contrasts_minimum: Vec<&SwatchColorContrast> = contrasts
            .iter()
            .filter(|contrast| {
                super::is_minimum_text_contrast(
                    swatch_a[contrast.swatch_a_idx],
                    swatch_b[contrast.swatch_b_idx],
                )
            })
            .collect::<Vec<_>>();

        let text_contrasts_enhanced = contrasts
            .iter()
            .filter(|contrast| {
                super::is_enhanced_text_contrast(
                    swatch_a[contrast.swatch_a_idx],
                    swatch_b[contrast.swatch_b_idx],
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(text_contrasts_minimum.len(), 51);
        assert_eq!(text_contrasts_enhanced.len(), 36);
    }
}
