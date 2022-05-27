use bevy::prelude::*;

use crate::points::Joint;

use super::*;

/// System to handle deletion of joints
/// 
/// *active
pub fn delete_joint(
    mut commands: Commands,
    key_input: Res<Input<KeyCode>>,
    mut joint_selected: ResMut<JointSelected>,
    joint_q: Query<&Joint>,
) {
    if joint_selected.0.is_none() || !key_input.just_pressed(KeyCode::Delete) {
        return;
    }

    let joint = joint_selected.0.unwrap();
    let joint_info = joint_q.get(joint).unwrap();

    if let Some(rotator) = joint_info.rotator {
        commands.entity(rotator).despawn();
    }
    if let Some(connector) = joint_info.connector {
        commands.entity(connector).despawn();
    }
    
    commands.entity(joint).despawn_recursive();
    joint_selected.0 = None;
}