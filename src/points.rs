use bevy::{prelude::*};
use bevy_mod_picking::{PickableMesh};
use serde::{Serialize, Deserialize};

use crate::{
    editor::{selection::*, Editable, EditMode}
};

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


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Point {
    pub r_coords: (f32, f32, f32),
    pub connections: Vec<Point>,
}

#[derive(Clone, Debug, Default, Component)]
pub struct Joint {
    pub dist: f32,
    pub parent: Option<Entity>,
    pub rotator: Option<Entity>,
    pub connector: Option<Entity>,   
}

#[derive(Component)]
pub struct Root;

impl Point {
    pub fn create_object(
        &mut self,
        commands: &mut Commands,
        meshes: &Res<JointMeshes>,
        materials: &Res<JointMaterial>,
        parent: Option<Entity>,
    ) {
        let joint = create_joint(
            parent, 
            Vec3::new(self.r_coords.0, self.r_coords.1, self.r_coords.2), 
            None,
            commands, 
            meshes, 
            materials
        );
        for connection in &mut self.connections {
            connection.create_object(commands, meshes.clone(), materials, Some(joint));
        }
    }
}

pub fn generate_mesh(
    mut commands: Commands,
    meshes: Res<JointMeshes>,
    materials: Res<JointMaterial>,
) {
    let point_data = &std::fs::read("./points.ron").unwrap();
    // let point_data = include_bytes!("../assets/points.ron");
    let mut points: Point = ron::de::from_bytes(point_data).unwrap_or_default();
    println!("{:?}", points);
    
    points.create_object(&mut commands, &meshes, &materials, None);
}

/// Creates a joint parented to the given entity (free floating if none) with the given
/// relative position (in global coordinate space). Joint will be initialized with the
/// given edit_mode if any.
pub fn create_joint(
    mut parent: Option<Entity>,
    position: Vec3,
    edit_mode: Option<EditMode>,
    commands: &mut Commands,
    meshes: &Res<JointMeshes>,
    materials: &Res<JointMaterial>,
) -> Entity {
    let mut joint = Joint::default();

    if parent.is_none() {
        parent = Some(commands.spawn_bundle(PbrBundle {
                mesh: meshes.head.clone(),
                material: materials.joint_color.clone(),
                transform: Transform::from_translation(position),
                ..Default::default()
            })
            .insert(PickableMesh::default())
            //.insert(BoundVol::default())
            .insert(Selectable::default())
            .insert(Editable::default())
            .insert(Root)
            .insert(joint)
            .id());
    } else {
        let len = position.length();
        let scale = Vec3::from([1.0, 1.0, 1.0]);
        let rotation = Quat::from_rotation_arc(Vec3::Y, position.normalize());
        
        let rotator = Some(commands.spawn_bundle(PbrBundle {
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, Vec3::default())),
            ..Default::default()
        })
          .with_children(|p| {
            // connector
            let scale = Vec3::from([1.0, len/2.0, 1.0]);
            let rotation = Quat::default();
            let translation = Vec3::new(0.0, len/2.0, 0.0);

            joint.connector = Some(p.spawn_bundle(PbrBundle {
                mesh: meshes.connector.clone(),
                material: materials.connector_color.clone(),
                transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, translation)),
                ..Default::default()
            }).id());
        }).id());
        joint.dist = len;
        joint.parent = parent;
        joint.rotator = rotator;

        let current = commands.spawn_bundle(PbrBundle {
            mesh: meshes.head.clone(),
            material: materials.joint_color.clone(),
            transform: Transform::from_translation(position),
            ..Default::default()
        })
        .insert(PickableMesh::default())
        //.insert(BoundVol::default())
        .insert(Selectable::default())
        .insert(Editable{ mode: edit_mode })
        .insert(joint)
        // .push_children(&[rotator.unwrap()])
        .id();

        commands.entity(parent.unwrap()).push_children(&[current, rotator.unwrap()]);
        parent = Some(current);
    }
    parent.unwrap()
}