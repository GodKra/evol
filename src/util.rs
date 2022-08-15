use bevy::{prelude::*};
use bevy_mod_raycast::Ray3d;

use std::hash::Hash;

pub struct JointMaterial {
    pub joint_color: Handle<StandardMaterial>,
    pub connector_color: Handle<StandardMaterial>,
    pub dof_color: Handle<StandardMaterial>,
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
            dof_color: materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 1.0, 0.0),
                unlit: true,
                ..Default::default()
            }),
        }
    }
}

pub struct JointMeshes {
    pub head: Handle<Mesh>,
    pub connector: Handle<Mesh>,
    pub dof_free: Handle<Mesh>,
    pub dof_locked: Handle<Mesh>,
}

impl FromWorld for JointMeshes {
    fn from_world(world: &mut World) -> Self {
        let (dof_locked, dof_free): (Handle<Mesh>, Handle<Mesh>) = {
            let asset_server = world.resource::<AssetServer>();
            (
                asset_server.load("models/dof_pointer.glb#Mesh0/Primitive0"),
                asset_server.load("models/dof_pointer.glb#Mesh1/Primitive0")
            )
        }; // because borrowchecker
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
            dof_locked, 
            dof_free 
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

/// Despawn all entities and their children with a given component type
pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

/// Gets the the point of intersection between a plane and ray. Both plane and ray should be in the same coordinate space.
pub fn get_intersect_plane_ray(plane_pos: Vec3, plane_normal: Vec3, ray: Ray3d) -> Vec3 {
    ray.origin() + ((plane_pos - ray.origin()).dot(plane_normal))/(ray.direction().dot(plane_normal)) * ray.direction()
}

/// Returns the quaternion needed for `src` to rotate around the `axis` to reach `dest`.
/// 
/// `src` and `dest` has to be orthogonal to the `axis`.
pub fn get_axis_rotation(src: Vec3, dest: Vec3, axis: Vec3) -> Quat {
    let angle = src.angle_between(dest);
    let cross = src.cross(dest);
    let rotation = if cross.dot(axis) > 0.0 {
        angle
    } else {
        std::f32::consts::TAU-angle
    };
    Quat::from_axis_angle(axis, rotation)
}