use std::collections::HashSet;

use bevy::prelude::*;
use petgraph::visit::EdgeRef;

use crate::{structure::*, selection::EntitySelected};

#[derive(Event)]
pub struct DeleteEvent;

/// System to handle deletion of joints, connectors, and muscles.
pub fn delete(
    _: Trigger<DeleteEvent>,
    mut commands: Commands,
    mut structure: ResMut<Structure>,
    mut entity_selected: ResMut<EntitySelected>,
    joint_q: Query<&mut Joint>,
    connector_q: Query<&Connector>,
    muscle_q: Query<&Muscle>,
) {
    if entity_selected.is_joint() { // delete joint and its relatives
        let joint = entity_selected.get().unwrap();
        let joint_info = joint_q.get(joint).unwrap().clone();

        let mut paired_edges = Vec::new(); // edges with muscles to edges that are deleted
        let mut completed_muscles: HashSet<Entity> = HashSet::new(); // muscles that have already been deleted
        let mut orphans = Vec::new(); // joints whose parents have potentially been deleted

        info!(":: Deleted Joint: {:?}", joint);

        for edge in structure.edges(joint_info.node_index) {
            let muscles = &edge.weight().muscles;
            for (opp_edge, muscle) in muscles {
                if completed_muscles.contains(muscle) {
                    continue;
                }
                paired_edges.push((*opp_edge, edge.id()));
                commands.entity(*muscle).despawn();
                completed_muscles.insert(*muscle);
            }
            let connector = structure.edge_to_entity(edge.id()).unwrap();
            commands.entity(connector).despawn();
            
            let endpoint = if edge.source() == joint_info.node_index {
                edge.target()
            } else {
                edge.source()
            };
            orphans.push(endpoint);
        }
        for (alive, dead) in paired_edges {
            let weight = structure.edge_weight_mut(alive).unwrap();
            weight.muscles.remove(&dead);
        }
        for orphan in orphans {
            let weight = structure.node_weight_mut(orphan).unwrap();
            if weight.parent == Some(joint_info.node_index) {
                weight.parent = None;
            }
        }
        commands.entity(joint).despawn();
        structure.remove_node(joint_info.node_index);

        entity_selected.set(None);
    } else if entity_selected.is_connector() { // delete connector
        let connector = entity_selected.get().unwrap();
        let conn_info = connector_q.get(connector).unwrap();
        let weight = structure.edge_weight(conn_info.edge_index).unwrap().clone();
        for (opp_edge, muscle) in weight.muscles.iter() {
            let opp_weight = structure.edge_weight_mut(*opp_edge).unwrap();
            opp_weight.muscles.remove(&conn_info.edge_index);
            commands.entity(*muscle).despawn();
        }
        let (j1, j2) = structure.edge_endpoints(conn_info.edge_index).unwrap();
        if structure.node_weight(j1).unwrap().parent == Some(j2) {
            structure.node_weight_mut(j1).unwrap().parent = None;
        }
        if structure.node_weight(j2).unwrap().parent == Some(j1) {
            structure.node_weight_mut(j2).unwrap().parent = None;
        }
        structure.remove_edge(conn_info.edge_index);
        info!(":: Deleted Joint: {:?}", connector);
        commands.entity(connector).despawn();
    } else if entity_selected.is_muscle() { // delete muscle
        let muscle = entity_selected.get().unwrap();
        let muscle_info = muscle_q.get(muscle).unwrap();
        let a1 = structure.edge_weight_mut(muscle_info.anchor1.unwrap()).unwrap();
        a1.muscles.remove(&muscle_info.anchor2.unwrap());
        let a2 = structure.edge_weight_mut(muscle_info.anchor2.unwrap()).unwrap();
        a2.muscles.remove(&muscle_info.anchor1.unwrap());

        commands.entity(muscle).despawn();
        info!(":: Deleted Muscle: {:?}", muscle);
        entity_selected.set(None);
    }
}