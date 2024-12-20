use bevy::{picking::pointer::PointerInteraction, prelude::*};

use super::controls::EditMode;
use crate::{
    editor::controls::ActionEvent, 
    selection::{EntitySelected, SelectableEntity}, 
    structure::*, 
    util::{JointMaterial, JointMeshes}
};

#[derive(Event)]
pub struct JointAddEvent;

#[derive(Event)]
pub struct JointLinkEvent;

/// System to handle the addition of new joints.
pub fn joint_add(
    mut commands: Commands,
    mut structure: ResMut<Structure>,
    mut entity_selected: ResMut<EntitySelected>,
    joint_materials: Res<JointMaterial>,
    joint_meshes: Res<JointMeshes>,
    edit_mode: Res<EditMode>,
    mut ev_joint_add: EventReader<JointAddEvent>,
    mut gizmo: Gizmos,
    joint_q: Query<&Joint>,
    interaction_q: Query<&PointerInteraction>,
) {
    let EditMode::JointAdd(joint) = *edit_mode else {
        return;
    };

    for interaction in interaction_q.iter() {
        let Some((target, hit)) = interaction.get_nearest_hit() else {
            return;
        };
        
        // Cancel if something other than the joint being edited is clicked.
        if *target != joint {
            if !ev_joint_add.is_empty() {
                ev_joint_add.clear(); 
                commands.send_event(super::controls::ActionEvent::Cancel);
            }
            return;
        }

        let hit_pos = hit.position.unwrap();
        let hit_normal = hit.normal.unwrap();
        
        gizmo.ray(hit_pos, hit_normal * hit.depth/4.0, Color::srgb(1.0, 0.0, 0.0));

        // Create the new joint where clicked.
        if !ev_joint_add.is_empty() {
            ev_joint_add.clear();

            let len = 2.0; // default extension

            let new_joint = create_joint(
                &mut commands, 
                &joint_meshes, 
                &joint_materials, 
                hit_pos + hit_normal * len, 
                None,
                (),
                crate::Editor
            );

            let parent_data = joint_q.get(joint).unwrap();

            let node = structure.add_node(
                Point { 
                    entityid: Some(new_joint), 
                    parent: Some(parent_data.node_index),
                    pos: hit_pos + hit_normal * len,
                }
            );
            commands.entity(new_joint).insert(Joint { node_index: node });

            let connector = create_connector(
                &mut commands, 
                &joint_meshes, 
                &joint_materials, 
                hit_pos + hit_normal * len, 
                hit_pos + hit_normal * -crate::util::JOINT_RADIUS, 
                None,
                (),
                crate::Editor
            );

            let edge = structure.add_edge(node, parent_data.node_index, Connection {
                entityid: Some(connector),
                ..default()
            });
            commands.entity(connector).insert(Connector{edge_index: edge},);
            
            commands.send_event(super::controls::ActionEvent::Cancel);
            commands.send_event(super::controls::ActionEvent::AdjustExtend);
            
            // Update selection.
            entity_selected.set(Some(SelectableEntity::Joint(new_joint)));
            commands.trigger(crate::selection::SelectionUpdateEvent);
            commands.send_event(crate::selection::SelectionBlockEvent);
        }
    }
}

pub fn joint_link(
    mut commands: Commands,
    mut structure: ResMut<Structure>,
    edit_mode: Res<EditMode>,
    entity_selected: Res<EntitySelected>,
    meshes: Res<JointMeshes>,
    materials: Res<JointMaterial>,
    mut ev_link: EventReader<JointLinkEvent>,
    joint_q: Query<&Joint>,
) {
    let EditMode::JointLink(entity) = *edit_mode else {
        return;
    };

    if ev_link.is_empty() {
        return;
    }

    ev_link.clear();
    
    let joint1 = joint_q.get(entity).unwrap();
    let joint2 = joint_q.get(entity_selected.get().unwrap()).unwrap();
    

    let j1 = joint1.node_index;
    let j2 = joint2.node_index;

    if j1 == j2 { // should not happen
        return;
    }

    structure.0.node_weight_mut(j1).unwrap().parent = Some(j2);

    if structure.0.contains_edge(j1, j2) {
        info!(":: Parent set: {:?} -> {:?}", j2, j1);
        commands.send_event(ActionEvent::Cancel);
        return;
    }

    let j1_pos = structure.0.node_weight(j1).unwrap().pos;
    let j2_pos = structure.0.node_weight(j2).unwrap().pos;

    let connector = create_connector(
        &mut commands, 
        &meshes, 
        &materials, 
        j1_pos,
        j2_pos,
        None,
        (),
        crate::Editor
    );

    let edge = structure.0.add_edge(j1, j2, Connection {
        entityid: Some(connector),
        ..default()
    });
    commands.entity(connector).insert(Connector{edge_index: edge});

    info!(":: Link created: {:?} <> {:?}", j1, j2);
}

pub fn update_structure_pos(
    mut structure: ResMut<Structure>,
    changed_q: Query<(&Joint, &Transform), Changed<Transform>>,
) {
    for (joint, transform) in changed_q.iter() {
        let weight = structure.node_weight_mut(joint.node_index).unwrap();
        weight.pos = transform.translation;
    }
}

// passive
/// automatically updates the rotation and scaling of the connectors when joint location
/// is updated
pub fn update_connector(
    structure: Res<Structure>,
    mut transform_set: ParamSet<(
        Query<(&Joint, &Transform), Changed<Transform>>,
        Query<&mut Transform>
    )>,
) {
    let mut changed_joints: Vec<(Joint, Vec3)> = Vec::new();
    for (joint, transform) in transform_set.p0().iter() {
        changed_joints.push((joint.clone(), transform.translation));
    }
    let mut transform_q = transform_set.p1();
    for (joint, j_translation) in changed_joints.iter() {
        let neighbors = structure.neighbors(joint.node_index);
        for neighbour in neighbors {
            let edge = structure.find_edge(joint.node_index, neighbour).unwrap();

            let parent = structure.node_to_entity(neighbour).unwrap();
            let connector = structure.edge_to_entity(edge).unwrap();

            let p_translation = transform_q.get(parent).unwrap().translation;
            let mut transform = transform_q.get_mut(connector).unwrap();

            let relative_pos = *j_translation-p_translation;
            
            let rotation = Quat::from_rotation_arc(Vec3::Y, relative_pos.normalize());
            let scale = Vec3::from([1.0, 1.0, 1.0]);
            let rotate = Mat4::from_scale_rotation_translation(scale, rotation, p_translation);
            
            let radius = relative_pos.length();
            let scale = Vec3::from([1.0, radius/2.0, 1.0]);
            let translation = Vec3::new(0.0, radius/2.0, 0.0);
            let position = Mat4::from_scale_rotation_translation(scale, Quat::default(), translation);

            *transform = Transform::from_matrix(rotate * position);

        }
    }
}