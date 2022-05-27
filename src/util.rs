use bevy::{prelude::*};
use bevy_mod_raycast::Ray3d;

pub fn ray_from_screenspace(
    cursor_pos_screen: Vec2,
    windows: &Windows,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Ray3d> {
    let view = camera_transform.compute_matrix();
    let windowid = if let bevy::render::camera::RenderTarget::Window(id) = camera.target {
        id
    } else {
        return None;
    };
    let window = match windows.get(windowid) {
        Some(window) => window,
        None => {
            error!("WindowId {} does not exist", windowid);
            return None;
        }
    };
    let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);
    let projection = camera.projection_matrix;

    // 2D Normalized device coordinate cursor position from (-1, -1) to (1, 1)
    let cursor_ndc = (cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
    let ndc_to_world: Mat4 = view * projection.inverse();
    let world_to_ndc = projection * view;
    let is_orthographic = projection.w_axis[3] == 1.0;

    // Compute the cursor position at the near plane. The bevy camera looks at -Z.
    let ndc_near = world_to_ndc.transform_point3(-Vec3::Z * camera.near).z;
    let cursor_pos_near = ndc_to_world.transform_point3(cursor_ndc.extend(ndc_near));

    // Compute the ray's direction depending on the projection used.
    let ray_direction = match is_orthographic {
        true => view.transform_vector3(-Vec3::Z), // All screenspace rays are parallel in ortho
        false => cursor_pos_near - camera_transform.translation, // Direction from camera to cursor
    };

    Some(Ray3d::new(cursor_pos_near, ray_direction))
}