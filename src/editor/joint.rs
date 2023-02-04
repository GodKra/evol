
use bevy::{prelude::*};
use petgraph::stable_graph::NodeIndex;

use crate::util::{JointMaterial, JointMeshes};

use super::{selection::*, IsLinkMode, pgraph::*};

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
            None
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