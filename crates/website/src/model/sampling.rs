use js_sys::Math::sqrt;

pub fn get_equidistant_points_in_range(start: f32, end: f32, count: usize) -> Vec<f32> {
    let mut points = vec![];

    for idx in 0..count {
        let t = idx as f32 / (count as f32 - 1.);
        let x = start + t * (end - start);

        points.push(x);
    }

    points
}

pub fn algebraic_simple(x: f64) -> f64 {
    x / sqrt(1. + x.powi(2))
}
