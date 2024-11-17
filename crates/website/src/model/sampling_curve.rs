use futures_signals::signal::Mutable;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use crate::model::palette_color::DWIND_CURVE2;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SamplingCurve {
    pub name: Mutable<String>,
    /// Samples in the color plane (0..1, 0..1)
    pub curve: Mutable<Vec<Vec2>>,
}

impl SamplingCurve {
    pub fn tailwind_happy() -> Self {
        Self {
            name: Mutable::new("Tailwind Happy".to_string()),
            curve: DWIND_CURVE2.into_iter().map(|v| Vec2::new(v.0, v.1)).collect::<Vec<_>>().into(),
        }
    }

    pub fn tailwind_diagonal() -> Self {
        Self {
            name: "Tailwind Diagonal".to_string().into(),
            curve: (0..11).into_iter().map(|v| {
                let x = v as f32 / 11.;

                Vec2::new(x, 1. - x)
            }).collect::<Vec<_>>().into(),
        }
    }
}