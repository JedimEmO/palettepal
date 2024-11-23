use futures_signals::signal::Mutable;
use glam::{Mat4, Vec2, Vec3, Vec4};

#[derive(Clone, Debug)]
pub struct Transform {
    pub screen_size: Vec2,
    pub viewport: Mat4,
    pub projection: Mat4,
    pub scale: Mat4,
    pub translate_screen: Vec2,
    pub dynamic_mat: Mutable<Mat4>,
}

#[derive(Copy, Clone, Debug)]
pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3) -> Self {
        Self { point, normal }
    }

    pub fn xy() -> Self {
        Self {
            point: Vec3::ZERO,
            normal: Vec3::Z,
        }
    }

    pub fn xz() -> Self {
        Self {
            point: Vec3::ZERO,
            normal: Vec3::Y,
        }
    }

    pub fn yz() -> Self {
        Self {
            point: Vec3::ZERO,
            normal: Vec3::X,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub corner: Vec2,
    pub dimension: Vec2,
}

impl AABB {
    pub fn new(x: f32, y: f32, x2: f32, y2: f32) -> Self {
        Self {
            corner: Vec2::new(x, y),
            dimension: Vec2::new(x2 - x, y2 - y),
        }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        let min = self.corner;
        let max = self.corner + self.dimension;

        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }
}

impl Transform {
    pub fn screen_to_clip_space(&self, screen_pos: Vec2) -> Vec2 {
        screen_pos - self.translate_screen
    }

    pub fn homogenous_to_world(&self, pos: Vec4) -> Vec3 {
        let mat = self.scale * self.projection * self.viewport;
        let inv = mat.inverse();
        inv.transform_vector3(Vec3::new(pos.x, pos.y, pos.z))
    }

    pub fn world_to_screen(&self, point: Vec3) -> Vec2 {
        let mat = self.viewport * self.projection * self.scale;
        mat.transform_vector3(point).truncate() + self.translate_screen
    }

    pub fn screen_to_world(&self, point: Vec2) -> Vec3 {
        let mat = self.scale * self.projection * self.viewport;
        let inv = mat.inverse();
        inv.transform_vector3(Vec3::new(point.x, point.y, 0.))
    }

    pub fn matrix(&self) -> Mat4 {
        self.scale * self.projection * self.viewport
    }

    // projection utilities
    pub fn project_screen_coords_to_world_plane(&self, screen: Vec2, plane: Plane) -> Vec3 {
        let ray_start_screen = self.screen_to_clip_space(screen).extend(0.);
        let ray_end_screen = self.screen_to_clip_space(screen).extend(1.);

        let ray_origin = self.homogenous_to_world(ray_start_screen.extend(1.));
        let ray_end = self.homogenous_to_world(ray_end_screen.extend(1.));
        let ray_normal = (ray_end - ray_origin).normalize();
        let t = (plane.point - ray_origin).dot(plane.normal) / (ray_normal.dot(plane.normal));

        ray_origin + ray_normal * t
    }

    pub fn project_screen_pos_on_plane(&self, screen: Vec2) -> Vec3 {
        self.project_screen_coords_to_world_plane(screen, Plane::new(Vec3::ZERO, Vec3::Z))
    }

    pub fn project_screen_pos_on_clipped_plane(
        &self,
        screen: Vec2,
        plane: Plane,
        aabb: AABB,
    ) -> Option<Vec2> {
        let plane_position = self.project_screen_coords_to_world_plane(screen, plane);

        if aabb.contains(plane_position.truncate()) {
            let clipped_position = plane_position.truncate() - aabb.corner;

            Some(clipped_position)
        } else {
            None
        }
    }

    pub fn brick_transform() -> Self {
        let screen_size = Vec2::new(512., 512.);
        let rot_mat = Mat4::from_rotation_y(45.);

        let mat = rot_mat;

        Self {
            viewport: Mat4::from_cols_array(&[
                1.,
                0.,
                0.,
                screen_size.x / 2.,
                0.,
                1.,
                0.,
                screen_size.y / 2.,
                0.,
                0.,
                1.,
                0.,
                0.,
                0.,
                0.,
                1.,
            ]) * Mat4::from_cols_array(&[
                screen_size.x / 2.,
                0.,
                0.,
                0.,
                0.,
                -screen_size.y / 2.,
                0.,
                0.,
                0.,
                0.,
                1.,
                0.,
                0.,
                0.,
                0.,
                1.,
            ]),
            projection: Mat4::look_at_lh(Vec3::new(0., 0.2, -0.3), Vec3::ZERO, Vec3::Y),
            scale: Mat4::from_scale(Vec3::new(0.6 * 1.5, 0.6, 0.6)),
            translate_screen: screen_size / 2.,
            screen_size,
            dynamic_mat: Mutable::new(Mat4::IDENTITY),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        let screen_size = Vec2::new(512., 512.);

        Self {
            viewport: Mat4::from_cols_array(&[
                1.,
                0.,
                0.,
                screen_size.x / 2.,
                0.,
                1.,
                0.,
                screen_size.y / 2.,
                0.,
                0.,
                1.,
                0.,
                0.,
                0.,
                0.,
                1.,
            ]) * Mat4::from_cols_array(&[
                screen_size.x / 2.,
                0.,
                0.,
                0.,
                0.,
                -screen_size.y / 2.,
                0.,
                0.,
                0.,
                0.,
                1.,
                0.,
                0.,
                0.,
                0.,
                1.,
            ]),
            projection: Mat4::look_at_lh(Vec3::new(0., 0.2, -0.3), Vec3::ZERO, Vec3::Y),
            scale: Mat4::from_scale(Vec3::new(0.6 * 1.5, 0.6, 0.6)),
            translate_screen: screen_size / 2.,
            screen_size,
            dynamic_mat: Mutable::new(Mat4::IDENTITY),
        }
    }
}
