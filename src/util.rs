use bevy::{prelude::*};

use core::fmt;
use std::hash::Hash;



pub enum Errors {
    /// Errors caused when attempting to get current window. Usually for mouse cursor.
    Window,
    /// Errors caused when a component is missing. (Component, Entity)
    ComponentMissing(&'static str, Entity),
    /// Errors caused when an element is missing from the ID map. (ID, EntityID)
    IDMapIncomplete(Option<u32>, Option<Entity>),
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Errors::Window => 
                write!(f, "WindowError: Not found"),
            Errors::ComponentMissing(component, entity) => 
                write!(f, "ComponentMissingError: Component {:?} not found for entity {:?}", component, entity),
            Errors::IDMapIncomplete(id, entity) => 
                write!(f, "IDMapIncompleteError: {:?} <> {:?}", id, entity),
        }
    }
}
// impl Into<String> for Errors {
//     fn into(self) -> String {
//         
//     }
// }

#[derive(Resource)]
pub struct JointMaterial {
    pub joint_color: Handle<StandardMaterial>,
    pub connector_color: Handle<StandardMaterial>,
    pub muscle_color: Handle<StandardMaterial>,
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
            muscle_color: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 0.0, 0.0),
                unlit: true,
                ..Default::default()
            }),
        }
    }
}

#[derive(Resource)]
pub struct JointMeshes {
    pub head: Handle<Mesh>,
    pub connector: Handle<Mesh>,
    pub muscle: Handle<Mesh>,
    // pub dof_free: Handle<Mesh>,
    // pub dof_locked: Handle<Mesh>,
}

impl FromWorld for JointMeshes {
    fn from_world(world: &mut World) -> Self {
        // let (dof_free, dof_locked): (Handle<Mesh>, Handle<Mesh>) = {
        //     let asset_server = world.resource::<AssetServer>();
        //     (
        //         asset_server.load("models/dof_pointer.glb#Mesh0/Primitive0"),
        //         asset_server.load("models/dof_pointer.glb#Mesh1/Primitive0")
        //     )
        // }; // because borrowchecker
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
            })),
            muscle: meshes.add(Mesh::from(shape::Cube {
                size: 0.2,
            })),
            // dof_locked, 
            // dof_free 
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
    // EROTATE,
    // EGRAB,
    // EEXTRUDE,
    // ESWITCH_MODE,
    // NONE,
}

impl KeyControls {
    pub fn code(&self) -> KeyCode {
        match self {
            Self::ESAVE => KeyCode::S,
            Self::EDELETE => KeyCode::Delete,
            // Self::EROTATE => KeyCode::R,
            // Self::EGRAB => KeyCode::G,
            // Self::EEXTRUDE => KeyCode::E,
            // Self::ESWITCH_MODE => KeyCode::Tab,
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

/// Despawn all entities and their children with a given component type
pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

/// Gets the the point of intersection between a plane and ray. Both plane and ray should be in the same coordinate space.
pub fn get_intersect_plane_ray(plane_pos: Vec3, plane_normal: Vec3, ray: Ray) -> Vec3 {
    ray.origin + ((plane_pos - ray.origin).dot(plane_normal))/(ray.direction.dot(plane_normal)) * ray.direction
}

/// Returns the quaternion needed for `src` to rotate around the `axis` to reach `dest`.
/// 
/// `src` and `dest` has to be orthogonal to the `axis`.
pub fn get_axis_rotation(src: Vec3, dest: Vec3, axis: Vec3) -> Quat {
    Quat::from_axis_angle(axis, get_rotation_angle(src, dest, axis))
}

/// Returns the angle needed for `src` to rotate around the `axis` to reach `dest`.
/// 
/// `src` and `dest` has to be orthogonal to the `axis`.
pub fn get_rotation_angle(src: Vec3, dest: Vec3, axis: Vec3) -> f32 {
    let angle = src.angle_between(dest);
    let cross = src.cross(dest);
    if cross.dot(axis) > 0.0 {
        angle
    } else {
        std::f32::consts::TAU-angle
    }
}