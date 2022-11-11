use bevy::{prelude::*};
use bevy_mod_picking::{PickableMesh};
use bimap::BiHashMap;
use serde::{Serialize, Deserialize};

use crate::{
    editor::{selection::*, Editable, EditMode}
};

use crate::util::{JointMaterial, JointMeshes};

use super::{muscle::{MuscleConnectors, MuscleHalfs, MuscleData, Muscle}};
// use super::dof::*;

/// Joint ID Counter. Used to simplify serialization of muscles by mapping a 
/// sequential ID to the in game Entity ID of joints.
#[derive(Default, Debug)]
pub struct IDCounter(u32);

impl IDCounter {
    pub fn get(&mut self) -> u32 {
        return self.0;
    }
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

/// Sequential ID to EntityID map. ID 0 is reserved for mouse anchor.
#[derive(Default, Debug)]
pub struct IDMap(pub BiHashMap<u32, Entity>);

/// Data representation of Joints.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Point {
    pub r_coords: (f32, f32, f32),
    pub connections: Vec<Point>,
}

/// In-game representation of Points.
#[derive(Clone, Debug, Default, Component)]
pub struct Joint {
    pub dist: f32,
    pub locked: bool,
    pub parent: Option<Entity>,
    pub rotator: Option<Entity>,
    pub connector: Option<Entity>,   
}

/// Connector identifier component with reference to it's head (end) joint.
#[derive(Component)]
pub struct Connector {
    pub head_joint: Entity
}

/// Identifier component for first joint.
#[derive(Component)]
pub struct Root;

impl Point {
    pub fn create_points(
        &mut self,
        commands: &mut Commands,
        meshes: &Res<JointMeshes>,
        materials: &Res<JointMaterial>,
        id_counter: &mut ResMut<IDCounter>,
        id_map: &mut ResMut<IDMap>,
        muscle_data: &MuscleData,
        muscle_halfs: &mut MuscleHalfs,
        parent: Option<Entity>,
    ) {
        let joint = create_joint(
            parent, 
            Vec3::new(self.r_coords.0, self.r_coords.1, self.r_coords.2), 
            None,
            commands, 
            meshes, 
            materials,
            id_map,
            id_counter,
        );
        
        let id = id_counter.get();
        let mut muscle_connectors = MuscleConnectors::default();

        if let Some(muscle_pairs) = muscle_data.pairs.get(&id) {
            for pair in muscle_pairs {
                // Create muscle entity
                let muscle = commands.spawn_bundle(PbrBundle {
                    mesh: meshes.connector.clone(),
                    material: materials.muscle_color.clone(),
                    ..Default::default()
                }).insert(Muscle { anchor1: id, anchor2: *pair }).id();

                muscle_connectors.pair.insert(*pair, muscle);
                
                // Add to half muscle map.
                if muscle_halfs.pairs.contains_key(pair) {
                    let pair = muscle_halfs.pairs.get_mut(pair).unwrap();
                    pair.push((id, muscle));
                } else {
                    muscle_halfs.pairs.insert(*pair, vec![(id, muscle)]);
                }
            }
        }

        if let Some(pairs) = muscle_halfs.pairs.remove(&id) {
            for pair in pairs { 
                muscle_connectors.pair.insert(pair.0, pair.1);
            }
        }

        commands.entity(joint).insert(muscle_connectors);


        for connection in &mut self.connections {
            connection.create_points(
                commands, 
                meshes.clone(), 
                materials,
                id_counter,
                id_map,
                muscle_data,
                muscle_halfs,
                Some(joint)
            );
        }
    }
}

/// Creates a joint parented to the given entity (free floating if none) with the given
/// relative position. Joint will be initialized with the given edit_mode if any.
pub fn create_joint(
    mut parent: Option<Entity>,
    position: Vec3,
    edit_mode: Option<EditMode>,
    commands: &mut Commands,
    meshes: &Res<JointMeshes>,
    materials: &Res<JointMaterial>,
    id_map: &mut ResMut<IDMap>,
    id_counter: &mut ResMut<IDCounter>,
) -> Entity {
    let mut joint = Joint::default();

    if parent.is_none() {
        parent = {
            let p = commands.spawn_bundle(PbrBundle {
                mesh: meshes.head.clone(),
                material: materials.joint_color.clone(),
                transform: Transform::from_translation(position),
                ..Default::default()
            })
            .insert(PickableMesh::default())
            .insert(Editable::default())
            .insert(Root) // Only the root needs this marker (change in future?)
            .insert(crate::Editor)
            .insert(joint)
            .id();
            commands.entity(p).insert(Selectable::with_type(SelectableEntity::Joint(p)));
            Some(p)
        };
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

            joint.connector = Some(
                p.spawn_bundle(PbrBundle {
                    mesh: meshes.connector.clone(),
                    material: materials.connector_color.clone(),
                    transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, translation)),
                    ..Default::default()
                })
                // .insert(Connector)
                .insert(PickableMesh::default())
                .id()
            );
        }).id());
        commands.entity(joint.connector.unwrap()).insert(Selectable::with_type(SelectableEntity::Connector(joint.connector.unwrap())));
        
        joint.dist = len;
        joint.parent = parent;
        joint.rotator = rotator;

        // Main Joint
        let current = commands.spawn_bundle(PbrBundle {
            mesh: meshes.head.clone(),
            material: materials.joint_color.clone(),
            transform: Transform::from_translation(position),
            ..Default::default()
        })
        .insert(PickableMesh::default())
        .insert(Editable{ mode: edit_mode })
        .insert(MuscleConnectors::default())
        .insert(joint.clone())
        .id();
        commands.entity(current).insert(Selectable::with_type(SelectableEntity::Joint(current)));
        commands.entity(joint.connector.unwrap()).insert(Connector { head_joint: current });

        
        commands.entity(parent.unwrap()).push_children(&[current, rotator.unwrap()]);
        parent = Some(current);
    }
    
    id_counter.increment();
    id_map.0.insert(id_counter.get(), parent.unwrap()); // parent here refers to the joint created in this function (technically it is the new parent)
    return parent.unwrap();
}