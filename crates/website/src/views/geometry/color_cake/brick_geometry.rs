use crate::views::geometry::shader_program::ColorSpaceVertex;
use glam::{Mat4, Vec3};

pub fn brick_triangles(angle: f32) -> Vec<ColorSpaceVertex> {
    let back_y = angle.sin();

    let _rot_mat = Mat4::from_rotation_y(45.);
    let _scale_mat = Mat4::from_scale(Vec3::new(0.8, 0.8, 0.8));

    let mat = Mat4::IDENTITY;

    let front_bottom_left = mat.transform_vector3(Vec3::new(1., -1., -1.));
    let front_bottom_right = mat.transform_vector3(Vec3::new(1., -1., 1.));
    let front_top_left = mat.transform_vector3(Vec3::new(1., 1., -1.));
    let front_top_right = mat.transform_vector3(Vec3::new(1., 1., 1.));
    let back_bottom_left = mat.transform_vector3(Vec3::new(-1., -1. + back_y, -1.));
    let back_top_left = mat.transform_vector3(Vec3::new(-1., 1. + back_y, -1.));
    let back_top_right = mat.transform_vector3(Vec3::new(-1., 1. + back_y, 1.));

    vec![
        // Left side
        // 1
        ColorSpaceVertex {
            pos: back_top_left.to_array(),
            hsx: [1. + back_y, 0., 1.],
        },
        ColorSpaceVertex {
            pos: back_bottom_left.to_array(),
            hsx: [1., 0., 0.],
        },
        ColorSpaceVertex {
            pos: front_bottom_left.to_array(),
            hsx: [0., 0., 0.],
        },
        // 2
        ColorSpaceVertex {
            pos: back_top_left.to_array(),
            hsx: [1. + back_y, 0., 1.],
        },
        ColorSpaceVertex {
            pos: front_bottom_left.to_array(),
            hsx: [0., 0., 0.],
        },
        ColorSpaceVertex {
            pos: front_top_left.to_array(),
            hsx: [0. + back_y, 0., 1.],
        },
        // Front
        // 1
        ColorSpaceVertex {
            pos: front_top_right.to_array(),
            hsx: [0. + back_y, 1., 1.],
        },
        ColorSpaceVertex {
            pos: front_top_left.to_array(),
            hsx: [0. + back_y, 0., 1.],
        },
        ColorSpaceVertex {
            pos: front_bottom_left.to_array(),
            hsx: [0., 0., 0.],
        },
        // 2
        ColorSpaceVertex {
            pos: front_top_right.to_array(),
            hsx: [0. + back_y, 1., 1.],
        },
        ColorSpaceVertex {
            pos: front_bottom_left.to_array(),
            hsx: [0., 0., 0.],
        },
        ColorSpaceVertex {
            pos: front_bottom_right.to_array(),
            hsx: [0., 0., 0.],
        },
        // Top
        // 1
        ColorSpaceVertex {
            pos: back_top_right.to_array(),
            hsx: [1. + back_y, 1., 1.],
        },
        ColorSpaceVertex {
            pos: back_top_left.to_array(),
            hsx: [1. + back_y, 0., 1.],
        },
        ColorSpaceVertex {
            pos: front_top_left.to_array(),
            hsx: [0. + back_y, 0., 1.],
        },
        //2
        ColorSpaceVertex {
            pos: back_top_right.to_array(),
            hsx: [1. + back_y, 1., 1.],
        },
        ColorSpaceVertex {
            pos: front_top_left.to_array(),
            hsx: [0. + back_y, 0., 1.],
        },
        ColorSpaceVertex {
            pos: front_top_right.to_array(),
            hsx: [0. + back_y, 1., 1.],
        },
    ]
}
