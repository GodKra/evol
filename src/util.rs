use core::fmt;
use bevy::prelude::*;
use petgraph::stable_graph::{NodeIndex, EdgeIndex};


pub const JOINT_RADIUS: f32 = 1.0;

pub enum Errors {
    /// Error: Component not found.
    ComponentMissing(&'static str, Entity),
    /// Error: NodeIndex not found in graph.
    NodeMissing(NodeIndex),
    /// Error: EdgeIndex not found in graph.
    EdgeMissing(EdgeIndex),
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Errors::ComponentMissing(component, entity) => 
                write!(f, "(ComponentMissing): Component {:?} not found for entity {:?}", component, entity),
            Errors::NodeMissing(node) => 
                write!(f, "(NodeMissing): Node {:?} not found in graph", node),
            Errors::EdgeMissing(edge) => 
                write!(f, "(EdgeMissing): Edge {:?} not found in graph", edge),
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
            joint_color: materials.add(Color::srgb(0.0, 0.0, 0.0,)),
            connector_color: materials.add(
                StandardMaterial {
                    base_color: Color::srgb(0.8, 0.8, 0.8,),
                    unlit: true,
                    ..default()
                }
            ),
            muscle_color: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
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
}

impl FromWorld for JointMeshes {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        JointMeshes {
            head: meshes.add(Sphere {
                radius: JOINT_RADIUS,
            }),
            connector: meshes.add(Capsule3d {
                half_length: 0.75,
                radius: 0.25,
                ..Default::default()
            }),
            muscle: meshes.add(Cuboid {
                half_size: Vec3::new(0.1, 0.1, 0.1),
            }),
        }
    }
}

/// Despawn all entities and their children with a given component type
pub fn despawn_all<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

/// Gets the the point of intersection between a plane and ray. Both plane and ray should be in the same coordinate space.
pub fn get_intersect_plane_ray(plane_pos: Vec3, plane_normal: Vec3, ray: Ray3d) -> Vec3 {
    ray.origin + ((plane_pos - ray.origin).dot(plane_normal))/(ray.direction.dot(plane_normal)) * Vec3::from(ray.direction)
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