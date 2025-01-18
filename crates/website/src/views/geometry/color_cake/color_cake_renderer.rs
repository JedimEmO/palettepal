use crate::model::palette_color::{CakeType, ColorSpace};
use crate::views::geometry::color_cake::brick_geometry::brick_triangles;
use crate::views::geometry::cylinder_geometry;
use crate::views::geometry::cylinder_geometry::make_cylinder;
use crate::views::geometry::shader_program::ShaderProgram;
use crate::views::geometry::transform::Transform;
use anyhow::anyhow;
use futures_signals::signal::Mutable;
use glam::{Mat4, Vec2};
use web_sys::WebGl2RenderingContext;

pub struct ColorCake {
    shader_program: ShaderProgram,
    transform: Transform,
    sample_curve: Mutable<Vec<Vec2>>,
}

impl ColorCake {
    pub fn new(context: &WebGl2RenderingContext) -> anyhow::Result<Self> {
        let mut sides = cylinder_geometry::cylinder_sides(0.);
        let mut top_disk = cylinder_geometry::cylinder_top(true, 0.);
        let mut bottom_disk = cylinder_geometry::cylinder_top(false, 0.);

        let mut vertices = vec![];

        vertices.append(&mut bottom_disk);
        vertices.append(&mut sides);
        vertices.append(&mut top_disk);

        let shader_program = ShaderProgram::new(
            context,
            include_str!("shaders/cake_vertex.glsl"),
            include_str!("shaders/cake_fragment.glsl"),
            vertices,
        )?;

        Ok(Self {
            shader_program,
            transform: Default::default(),
            sample_curve: Default::default(),
        })
    }

    pub fn draw(
        &mut self,
        context: &WebGl2RenderingContext,
        hue: f32,
        color_space: ColorSpace,
        sample_points: Vec<Vec2>,
        cake_type: CakeType,
        plane_angle: f32,
    ) -> anyhow::Result<()> {
        self.sample_curve.set(sample_points.clone());
        let program = &self.shader_program.program;

        context.use_program(Some(program));

        let position_location = context.get_attrib_location(program, "a_position");
        let color_location = context.get_attrib_location(program, "a_color");
        let hue_location = context.get_uniform_location(program, "u_hue");
        let space_location = context.get_uniform_location(program, "u_space");
        let matrix_location = context.get_uniform_location(program, "u_matrix");

        let buffer = context
            .create_buffer()
            .ok_or(anyhow!("failed to create buffer"))?;

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        let mut vertices = match cake_type {
            CakeType::Cylinder => make_cylinder(plane_angle.to_radians()),
            CakeType::Brick => brick_triangles(plane_angle.to_radians()),
        };

        unsafe {
            let data_view = js_sys::Float32Array::view_mut_raw(
                vertices.as_mut_ptr() as *mut f32,
                vertices.len() * 6,
            );

            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &data_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        let vertex_array_object = context
            .create_vertex_array()
            .ok_or(anyhow!("failed to create vertex array object"))?;

        context.bind_vertex_array(Some(&vertex_array_object));

        context.vertex_attrib_pointer_with_i32(
            position_location as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            6 * size_of::<f32>() as i32,
            0,
        );

        context.vertex_attrib_pointer_with_i32(
            color_location as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            6 * size_of::<f32>() as i32,
            3 * size_of::<f32>() as i32,
        );

        context.enable_vertex_attrib_array(position_location as u32);
        context.enable_vertex_attrib_array(color_location as u32);

        let color_space = match color_space {
            ColorSpace::HSV => 0,
            ColorSpace::HSL => 1,
        };

        let scale = self.transform.scale;
        let view_matrix = self.transform.projection;

        let mut matrix = scale * view_matrix;

        if cake_type == CakeType::Brick {
            matrix *= Mat4::from_rotation_y(45.);
        }

        WebGl2RenderingContext::uniform1f(context, hue_location.as_ref(), hue);
        WebGl2RenderingContext::uniform1i(context, space_location.as_ref(), color_space);
        WebGl2RenderingContext::uniform_matrix4fv_with_f32_array(
            context,
            matrix_location.as_ref(),
            false,
            matrix.as_ref(),
        );

        context.clear_color(0.0, 0.0, 0.0, 0.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        context.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        context.clear_depth(0.);

        context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vertices.len() as i32);

        Ok(())
    }
}
