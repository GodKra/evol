use bevy::{prelude::*, ecs::schedule::IntoSystemDescriptor};
use bevy_mod_raycast::Ray3d;

use std::hash::Hash;

pub struct JointMaterial {
    pub joint_color: Handle<StandardMaterial>,
    pub connector_color: Handle<StandardMaterial>,
}

impl FromWorld for JointMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        JointMaterial {
            joint_color: materials.add(Color::rgb(0.0, 0.0, 0.0,).into()),
            connector_color: materials.add(
                StandardMaterial {
                    base_color: Color::rgb(0.8, 0.8, 0.8,),
                    emissive: Color::rgba_linear(0.6, 0.6, 0.6, 0.0),
                    ..default()
                }
            ),
        }
    }
}

pub struct JointMeshes {
    pub head: Handle<Mesh>,
    pub connector: Handle<Mesh>,
}

impl FromWorld for JointMeshes {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        JointMeshes {
            head: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.,
                subdivisions: 32,
            })),
            connector: meshes.add(Mesh::from(shape::Capsule {
                depth: 1.5,
                radius: 0.25,
                ..Default::default()
            }))
        }
    }
}

pub trait Control
{
    type I: 'static + Hash + Eq + Sync + Send + Copy;

    fn pressed(&self, kbd: Res<Input<Self::I>>) -> bool;
}

pub enum KeyControls {
    ESAVE,
    EDELETE,
    EROTATE,
    EGRAB,
    EEXTRUDE,
    ESWITCH_MODE,
    // NONE,
}

impl KeyControls {
    pub fn code(&self) -> KeyCode {
        match self {
            Self::ESAVE => KeyCode::S,
            Self::EDELETE => KeyCode::Delete,
            Self::EROTATE => KeyCode::R,
            Self::EGRAB => KeyCode::G,
            Self::EEXTRUDE => KeyCode::E,
            Self::ESWITCH_MODE => KeyCode::Tab,
        }
    }
}

impl Control for KeyControls {
    type I = KeyCode;
    fn pressed(&self, kbd: Res<Input<Self::I>>) -> bool {
        kbd.just_pressed(self.code())
    }
}

pub enum MouseControls {
    EINTERACT,
}

impl MouseControls {
    pub fn to_code(&self) -> MouseButton {
        match self {
            Self::EINTERACT => MouseButton::Left,
        }
    }
}


impl Control for MouseControls {
    type I = MouseButton;
    fn pressed(&self, kbd: Res<Input<Self::I>>) -> bool {
        kbd.just_pressed(self.to_code())
    }
}

/// [Ray3d]::from_screenspace modified to not require [Res]\<[Windows]\> for 
/// borrowchecker purposes.
pub fn ray_from_screenspace(
    cursor_pos_screen: Vec2,
    windows: &Windows,
    images: &Res<Assets<Image>>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Ray3d> {
    let view = camera_transform.compute_matrix();
    let screen_size = match camera.target.get_logical_size(windows, images) {
        Some(s) => s,
        None => {
            error!(
                "Unable to get screen size for RenderTarget {:?}",
                camera.target
            );
            return None;
        }
    };
    let projection = camera.projection_matrix;

    // 2D Normalized device coordinate cursor position from (-1, -1) to (1, 1)
    let cursor_ndc = (cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
    let ndc_to_world: Mat4 = view * projection.inverse();
    let world_to_ndc = projection * view;
    let is_orthographic = projection.w_axis[3] == 1.0;

    // Calculate the camera's near plane using the projection matrix
    let projection = projection.to_cols_array_2d();
    let camera_near = (2.0 * projection[3][2]) / (2.0 * projection[2][2] - 2.0);

    // Compute the cursor position at the near plane. The bevy camera looks at -Z.
    let ndc_near = world_to_ndc.transform_point3(-Vec3::Z * camera_near).z;
    let cursor_pos_near = ndc_to_world.transform_point3(cursor_ndc.extend(ndc_near));

    // Compute the ray's direction depending on the projection used.
    let ray_direction = match is_orthographic {
        true => view.transform_vector3(-Vec3::Z), // All screenspace rays are parallel in ortho
        false => cursor_pos_near - camera_transform.translation, // Direction from camera to cursor
    };

    Some(Ray3d::new(cursor_pos_near, ray_direction))
}

/// Despawn all entities and their children with a given component type
pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}