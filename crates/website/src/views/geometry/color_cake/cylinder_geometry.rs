use crate::views::geometry::shader_program::ColorSpaceVertex;
use std::f32::consts::PI;

pub fn make_cylinder(angle: f32) -> Vec<ColorSpaceVertex> {
    let mut out = vec![];

    let mut top = cylinder_top(true, angle);
    let mut sides = cylinder_sides(angle);

    out.append(&mut top);
    out.append(&mut sides);

    out
}

pub fn cylinder_top(top: bool, angle_offset: f32) -> Vec<ColorSpaceVertex> {
    let y = if top { 1. } else { -1. };
    let l = (y + 1.) / 2.;
    let angle_offset = if top { angle_offset } else { 0. };

    let mut out = vec![];

    let num_verts = 32;
    let slice_radius = (3. * PI / 2.) / num_verts as f32;
    let start_angle = 3. * PI / 2.;
    let pct = start_angle / (PI * 2.);

    for sector in 0..num_verts {
        let angle = start_angle - (sector as f32 * slice_radius);
        let next_angle = start_angle - ((sector + 1) as f32 * slice_radius);

        let h = (angle - angle_offset) / (2. * PI) / pct;
        let next_h = (next_angle - angle_offset) / (2. * PI) / pct;

        let x = angle.cos() * 1.;
        let z = angle.sin() * 1.;
        let next_x = next_angle.cos() * 1.;
        let next_z = next_angle.sin() * 1.;

        out.push(ColorSpaceVertex {
            pos: [0., y, 0.],
            hsx: [next_h, 0., l],
        });

        out.push(ColorSpaceVertex {
            pos: [next_x, y, next_z],
            hsx: [next_h, 1., l],
        });

        out.push(ColorSpaceVertex {
            pos: [x, y, z],
            hsx: [h, 1., l],
        });
    }

    out
}

pub fn cylinder_sides(angle_offset: f32) -> Vec<ColorSpaceVertex> {
    let mut out = vec![];

    let num_verts = 32;
    let slice_radius = (3. * PI / 2.) / num_verts as f32;
    let start_angle = 3. * PI / 2.;
    let pct = start_angle / (PI * 2.);

    let slice_h_top = -angle_offset / (2. * PI);

    for sector in 0..num_verts {
        let angle = start_angle - (sector as f32 * slice_radius);
        let next_angle = start_angle - ((sector + 1) as f32 * slice_radius);

        let h = angle / (2. * PI) / pct;
        let h_top = (angle - angle_offset) / (2. * PI) / pct;
        let next_h = next_angle / (2. * PI) / pct;
        let next_h_top = (next_angle - angle_offset) / (2. * PI) / pct;

        let x = angle.cos() * 1.;
        let z = angle.sin() * 1.;
        let next_x = next_angle.cos() * 1.;
        let next_z = next_angle.sin() * 1.;

        // Cylinder side triangles

        //A
        out.push(ColorSpaceVertex {
            pos: [x, 1., z],
            hsx: [h_top, 1., 1.0],
        });

        out.push(ColorSpaceVertex {
            pos: [next_x, -1., next_z],
            hsx: [next_h, 1., 0.],
        });

        out.push(ColorSpaceVertex {
            pos: [x, -1., z],
            hsx: [h, 1., 0.],
        });

        // B
        out.push(ColorSpaceVertex {
            pos: [next_x, -1., next_z],
            hsx: [next_h, 1., 0.],
        });

        out.push(ColorSpaceVertex {
            pos: [x, 1., z],
            hsx: [h_top, 1., 1.0],
        });

        out.push(ColorSpaceVertex {
            pos: [next_x, 1., next_z],
            hsx: [next_h_top, 1., 1.0],
        });
    }

    // Cylinder slice
    // A
    out.push(ColorSpaceVertex {
        pos: [1., 1., 0.],
        hsx: [slice_h_top, 1., 1.],
    });

    out.push(ColorSpaceVertex {
        pos: [0., 1., 0.],
        hsx: [slice_h_top, 0., 1.],
    });

    out.push(ColorSpaceVertex {
        pos: [0., -1., 0.],
        hsx: [0., 0., 0.],
    });

    // B
    out.push(ColorSpaceVertex {
        pos: [1., 1., 0.],
        hsx: [slice_h_top, 1., 1.],
    });

    out.push(ColorSpaceVertex {
        pos: [0., -1., 0.],
        hsx: [0., 0., 0.],
    });

    out.push(ColorSpaceVertex {
        pos: [1., -1., 0.],
        hsx: [0., 1., 0.],
    });

    out
}
