use crate::model::palette_color::DWIND_CURVE2;
use futures_signals::signal::Mutable;
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SamplingCurve {
    pub name: Mutable<String>,
    /// Samples in the color plane (0..1, 0..1)
    pub curve: Mutable<Vec<Vec2>>,
}

pub struct Modifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
}

impl SamplingCurve {
    pub fn add_new_point(&self, point: Vec2) {
        self.curve.lock_mut().push(point);
        self.sort();
    }

    pub fn try_y_align_at_x(&self, pos: Vec2) {
        let mut curve_data = self.curve.lock_mut();

        for point in curve_data.iter_mut() {
            if (point.x.abs() - pos.x.abs()).abs() < 0.01 {
                point.y = pos.y;

                return;
            }
        }
    }

    pub fn replace_point(&self, idx: usize, mut new_point: Vec2, modifiers: Modifiers) -> usize {
        {
            let mut curve_data = self.curve.lock_mut();

            new_point.x = if modifiers.alt {
                curve_data[idx].x
            } else {
                new_point.x
            };

            new_point.y = if modifiers.ctrl {
                curve_data[idx].y
            } else {
                new_point.y
            };

            curve_data[idx] = new_point;
        }

        self.sort();

        let new_idx = self
            .curve
            .lock_ref()
            .iter()
            .position(|v| v == &new_point)
            .unwrap();

        new_idx
    }

    pub fn sort(&self) {
        self.curve
            .lock_mut()
            .sort_by(|lhs, rhs| {
                if rhs.x.max(lhs.x) - rhs.x.min(lhs.x) < 0.0001 {
                    return rhs.y.total_cmp(&lhs.y)
                }

                lhs.x.total_cmp(&rhs.x)
            })
    }
}

impl SamplingCurve {
    pub fn new() -> Self {
        Self {
            name: Mutable::new("New Curve".to_string()),
            curve: vec![Vec2::new(0.5, 0.5)].into(),
        }
    }

    pub fn tailwind_happy() -> Self {
        Self {
            name: Mutable::new("Tailwind Happy".to_string()),
            curve: DWIND_CURVE2
                .into_iter()
                .map(|v| Vec2::new(v.0, v.1))
                .collect::<Vec<_>>()
                .into(),
        }
    }

    pub fn tailwind_diagonal() -> Self {
        Self {
            name: "Tailwind Diagonal".to_string().into(),
            curve: (0..11)
                .map(|v| {
                    let x = v as f32 / 11.;

                    Vec2::new(x, 1. - x)
                })
                .collect::<Vec<_>>()
                .into(),
        }
    }

    pub fn pixelart_5() -> Self {
        Self {
            name: "Pixelart 5".to_string().into(),
            curve: vec![
                Vec2::new(0.0, 1.0),
                Vec2::new(0.25, 0.8),
                Vec2::new(0.5, 0.6),
                Vec2::new(0.75, 0.30),
                Vec2::new(1.0, 0.0),
            ]
            .into(),
        }
    }
}
