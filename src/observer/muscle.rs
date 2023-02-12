use bevy::{prelude::*};

use crate::pgraph::*;


/// Updates the transform of the muscles when they have been created or the anchors have been moved.
pub fn update_muscles(
    pgraph: Res<PGraph>,
    mut muscle_set: ParamSet<(
        Query<(&Muscle, &mut Transform), (Added<Muscle>, Without<Joint>)>,
        Query<(&Muscle, &mut Transform), (With<Muscle>, Without<Joint>)>,
    )>,
    changed_joints: Query<&Joint, Changed<Transform>>,
    transform_q: Query<&Transform, (Without<Muscle>, Without<Joint>)>,
) {
    for (muscle, mut m_transform) in muscle_set.p0().iter_mut() { // update position on start 
        if muscle.anchor1 == None || muscle.anchor2 == None {
            return
        }

        let c1 = pgraph.edge_to_entity(muscle.anchor1.unwrap()).unwrap();
        let c2 = pgraph.edge_to_entity(muscle.anchor2.unwrap()).unwrap();

        *m_transform = get_muscle_transform(&transform_q, c1, c2,);
    }

    for joint in changed_joints.iter() {
        let edges = pgraph.0.edges(joint.node_index);
        for edge in edges {
            let e_weight = edge.weight();
            for (_, muscle) in e_weight.muscles.iter() {
                let mut query = muscle_set.p1();
                let (m_data, mut m_transform) = query.get_mut(*muscle).unwrap();
                let c1 = pgraph.edge_to_entity(m_data.anchor1.unwrap()).unwrap();
                let c2 = pgraph.edge_to_entity(m_data.anchor2.unwrap()).unwrap();

                *m_transform = get_muscle_transform(&transform_q, c1, c2,);
            }
        }
    }
}

/// Returns the muscle transform given the two anchor joints.
fn get_muscle_transform(
    transform_q: &Query<&Transform, (Without<Muscle>, Without<Joint>)>,
    connector1: Entity,
    connector2: Entity,
) -> Transform {
    let c1 = transform_q.get(connector1).unwrap().translation;
    let c2 = transform_q.get(connector2).unwrap().translation;

    let translation = (c1+c2) / 2.0;
    let scale = Vec3::new(0.5, c1.distance(c2) / 2.0, 0.5) ;
    let rotation = Quat::from_rotation_arc(Vec3::Y, (c1-translation).normalize());
    Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, translation))
}