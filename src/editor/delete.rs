use bevy::{prelude::*};

use super::{selection::EntitySelected, joint::*, muscle::{Muscle, MuscleConnectors}};

/// System to handle deletion of joints and muscles
/// 
/// *active
pub fn delete(
    mut commands: Commands,
    mut entity_selected: ResMut<EntitySelected>,
    id_map: Res<IDMap>,
    joint_q: Query<&Joint>,
    child_q: Query<&Children>,
    muscle_q: Query<&Muscle>,
    mut muscle_con_q: Query<&mut MuscleConnectors>,
) {
    if entity_selected.is_joint() { // delete joint
        let joint = entity_selected.get().unwrap();
        let joint_info = joint_q.get(joint).unwrap();
        delete_joints_recursive(&joint, joint_info, &mut commands, &id_map, &joint_q, &child_q, &mut muscle_con_q);
        println!(":: Deleted Joint(s): {:?} ..", joint);
        entity_selected.set(None);
    } else if entity_selected.is_muscle() { // delete muscle
        let muscle = entity_selected.get().unwrap();
        let muscle_info = muscle_q.get(muscle).unwrap();
        let a1 = id_map.0.get_by_left(&muscle_info.anchor1).unwrap();
        let a2 = id_map.0.get_by_left(&muscle_info.anchor2).unwrap();

        let mut a1_m = muscle_con_q.get_mut(*a1).unwrap();
        a1_m.pair.remove(&muscle_info.anchor2);
        
        let mut a2_m = muscle_con_q.get_mut(*a2).unwrap();
        a2_m.pair.remove(&muscle_info.anchor1);

        commands.entity(muscle).despawn();
        println!(":: Deleted Muscle: {:?}", muscle);
        entity_selected.set(None);
    }
}

/// deletes a joint and all entities relying on the existence of the joint (muscles, connectors, rotators, and children joints)
fn delete_joints_recursive(
    joint: &Entity,
    joint_info: &Joint,
    commands: &mut Commands,
    id_map: &Res<IDMap>,
    joint_q: &Query<&Joint>,
    child_q: &Query<&Children>,
    muscle_con_q: &mut Query<&mut MuscleConnectors>,
) {
    let muscle_info = muscle_con_q.get(*joint).unwrap();

    for (id, muscle) in muscle_info.pair.clone().iter() { // clone workaround for borrowchecker
        commands.entity(*muscle).despawn();
        let pair_joint = id_map.0.get_by_left(id).unwrap();
        let mut pair_info = muscle_con_q.get_mut(*pair_joint).unwrap();
        pair_info.pair.remove(id_map.0.get_by_right(joint).unwrap());
    }

    if let Some(rotator) = joint_info.rotator {
        commands.entity(rotator).despawn();
    }
    if let Some(connector) = joint_info.connector {
        commands.entity(connector).despawn();
    }

    commands.entity(*joint).despawn();

    let Ok(children) = child_q.get(*joint) else {
        return;
    };

    for child in children.iter() {
        if let Ok(joint_info) = joint_q.get(*child) {
            delete_joints_recursive(child, joint_info, commands, id_map, joint_q, child_q, muscle_con_q);
        }
    }
}