
use bevy::{prelude::*};
use petgraph::stable_graph::NodeIndex;

use crate::{util::{JointMaterial, JointMeshes}, selection::EntitySelected};

use super::IsLinkMode;
use crate::pgraph::*;

/// (Joint, Muscle); describes the first anchor when creating a muscle.
#[derive(Default, Resource)]
pub struct LinkRoot(Option<NodeIndex>);

pub fn link_joint (
    mut pgraph: ResMut<PGraph>,
    mut commands: Commands,
    mut is_link_mode: ResMut<IsLinkMode>,
    mut link_root: ResMut<LinkRoot>,
    entity_selected: Res<EntitySelected>,
    meshes: Res<JointMeshes>,
    materials: Res<JointMaterial>,
    key_input: Res<Input<KeyCode>>,
    joint_q: Query<&Joint>,
) {
    if !is_link_mode.0 && entity_selected.is_joint() && key_input.just_pressed(KeyCode::J) { // Link joint is set
        is_link_mode.0 = true;
        return;
    }
    if !is_link_mode.0 || !entity_selected.is_joint() { // Entity selected not a joint so reset
        if is_link_mode.0 {
            is_link_mode.0 = false;
            link_root.0 = None;
        }
        return;
    }

    let Ok(joint) = joint_q.get(entity_selected.get().unwrap()) else {
        return
    };
    if link_root.0.is_none() {
        link_root.0 = Some(joint.node_index);
    } else {
        let j1 = link_root.0.unwrap();
        let j2 = joint.node_index;

        if j1 == j2 {
            return;
        }

        pgraph.0.node_weight_mut(j1).unwrap().parent = Some(j2);

        if pgraph.0.contains_edge(j1, j2) {
            is_link_mode.0 = false;
            link_root.0 = None;
            println!(":: Parent set: {:?} -> {:?}", j2, j1);
            return;
        }

        let j1_pos = pgraph.0.node_weight(j1).unwrap().pos;
        let j2_pos = pgraph.0.node_weight(j2).unwrap().pos;

        let connector = create_connector(
            &mut commands, 
            &meshes, 
            &materials, 
            j1_pos,
            j2_pos,
            None,
            crate::Editor
        );

        let edge = pgraph.0.add_edge(j1, j2, Connection {
            entityid: Some(connector),
            ..default()
        });
        commands.entity(connector).insert(Connector{edge_index: edge});

        is_link_mode.0 = false;
        link_root.0 = None;
        println!(":: Link created: {:?} <> {:?}", j1, j2);
    }

}

pub fn update_pgraph_pos(
    mut pgraph: ResMut<PGraph>,
    changed_q: Query<(&Joint, &Transform), Changed<Transform>>,
) {
    for (joint, transform) in changed_q.iter() {
        let weight = pgraph.0.node_weight_mut(joint.node_index).unwrap();
        weight.pos = transform.translation;
    }
}

// passive
/// automatically updates the rotation and scaling of the connectors when joint location
/// is updated
pub fn update_connector(
    pgraph: Res<PGraph>,
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
        let neighbors = pgraph.0.neighbors(joint.node_index);
        for neighbour in neighbors {
            let edge = pgraph.0.find_edge(joint.node_index, neighbour).unwrap();

            let parent = pgraph.node_to_entity(neighbour).unwrap();
            let connector = pgraph.edge_to_entity(edge).unwrap();

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