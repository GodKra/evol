use bevy::{prelude::*};
use bevy_mod_picking::{PickableMesh};
use serde::{Serialize, Deserialize};

use crate::{
    editor::{selection::*, Editable, EditMode}
};

use crate::util::{JointMaterial, JointMeshes};
use super::dof::*;


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Point {
    pub r_coords: (f32, f32, f32),
    pub dof: f32, // Angle describing which direction the joint can move-- If zero then free
    pub connections: Vec<Point>,
}

/// Physical manifestation of the [Point] struct.
#[derive(Clone, Debug, Default, Component)]
pub struct Joint {
    pub dist: f32,
    pub dof: f32,
    pub locked: bool,
    pub parent: Option<Entity>,
    pub rotator: Option<Entity>,
    pub connector: Option<Entity>,   
    pub dof_pointer: Option<Entity>,
}

impl Joint {
    pub fn with_dof(dof: f32) -> Self {
        if dof == 0.0 {
            Joint { dof, locked: false, ..default() }
        } else {
            Joint { dof, locked: true, ..default() }
        }
    }
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
            self.dof,
            None,
            commands, 
            meshes, 
            materials,
        );
        for connection in &mut self.connections {
            connection.create_object(commands, meshes.clone(), materials,  Some(joint));
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
    
    points.create_object(&mut commands, &meshes, &materials,  None);
}

/// Creates a joint parented to the given entity (free floating if none) with the given
/// relative position. Joint will be initialized with the
/// given edit_mode if any.
pub fn create_joint(
    mut parent: Option<Entity>,
    position: Vec3,
    dof: f32,
    edit_mode: Option<EditMode>,
    commands: &mut Commands,
    meshes: &Res<JointMeshes>,
    materials: &Res<JointMaterial>,
) -> Entity {
    let mut joint = Joint::with_dof(dof);

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
            .insert(Root) // Only the root needs this marker (change in future?)
            .insert(crate::Editor)
            .insert(joint)
            .id());
    } else {
        let len = position.length();
        let scale = Vec3::from([1.0, 1.0, 1.0]);
        let rotation = Quat::from_rotation_arc(Vec3::Y, position.normalize());
        
        // Rotator
        let rotator = Some(commands.spawn_bundle(PbrBundle {
            transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, Vec3::default())),
            ..Default::default()
        })
          .with_children(|p| {
            // Connector
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

        // DOF Pointer
        let dof_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
            .with_scale(Vec3::new(DOF_SCALE, DOF_SCALE, DOF_SCALE));
        let pointer = commands.spawn_bundle(PbrBundle {
            mesh: if joint.locked { 
                    meshes.dof_locked.clone() 
                } else {
                    meshes.dof_free.clone() 
                },
            material: materials.dof_color.clone(),
            transform: dof_transform,
            visibility: Visibility { is_visible: false },
            ..default()
        }).insert(super::dof::DOFPointer)
        .insert(crate::Editor)
        .id();

        
        joint.dist = len;
        joint.parent = parent;
        joint.rotator = rotator;
        joint.dof_pointer = Some(pointer);

        // Main Joint
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
        .push_children(&[pointer])
        .id();

        commands.entity(parent.unwrap()).push_children(&[current, rotator.unwrap()]);
        parent = Some(current);
    }
    parent.unwrap()
}