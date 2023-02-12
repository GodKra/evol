
use bevy::{prelude::*};

use crate::pgraph::*;


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