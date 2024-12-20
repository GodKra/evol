use bevy::prelude::*;

use crate::{
    editor::controls::ActionEvent, 
    selection::EntitySelected, 
    structure::*, 
    util::{JointMaterial, JointMeshes}
};

use super::controls::EditMode;

#[derive(Event)]
pub struct MuscleAddEvent;

/// Creates muscles between two connectors.
pub fn muscle_construct(
    mut structure: ResMut<Structure>,
    mut commands: Commands,
    edit_mode: Res<EditMode>,
    entity_selected: Res<EntitySelected>,
    meshes: Res<JointMeshes>,
    materials: Res<JointMaterial>,
    mut ev_add: EventReader<MuscleAddEvent>,
    connector_q: Query<&Connector>,
) {
    let EditMode::MuscleAdd(entity) = *edit_mode else {
        return;
    };

    if ev_add.is_empty() {
        return;
    }

    ev_add.clear();

    let connector1 = connector_q.get(entity).unwrap();
    let connector2 = connector_q.get(entity_selected.get().unwrap()).unwrap();

    let anchor1 = connector1.edge_index;
    let anchor2 = connector2.edge_index;

    let anchor1_data = structure.edge_weight_mut(anchor1).unwrap();

    if anchor1_data.muscles.contains_key(&anchor2) {
        info!(":: Muscle already exists between edge {:?} and {:?}", anchor1, anchor2);
        commands.send_event(ActionEvent::Cancel);
        return;
    }

    let muscle = create_muscle(
        &mut commands, 
        &meshes, 
        &materials, 
        Some(anchor1), 
        Some(anchor2), 
        (), 
        crate::Editor
    );

    anchor1_data.muscles.insert(anchor2, muscle);

    let anchor2_data = structure.edge_weight_mut(anchor2).unwrap();
    anchor2_data.muscles.insert(anchor1, muscle);

    info!(":: Muscle Constructed: {:?} <> {:?}", anchor1, anchor2);
}

/// Updates the transform of the muscles when they have been created or the anchors have been moved.
pub fn update_muscles(
    structure: Res<Structure>,
    mut muscle_set: ParamSet<(
        Query<(&Muscle, &mut Transform), (Changed<Muscle>, Without<Joint>)>,
        Query<(&Muscle, &mut Transform), (With<Muscle>, Without<Joint>)>,
    )>,
    changed_joints: Query<&Joint, Changed<Transform>>,
    transform_q: Query<&Transform, (Without<Muscle>, Without<Joint>)>,
) {
    for joint in changed_joints.iter() {
        let edges = structure.edges(joint.node_index);
        for edge in edges {
            let e_weight = edge.weight();
            for (_, muscle) in e_weight.muscles.iter() {
                let mut query = muscle_set.p1();
                let (m_data, mut m_transform) = query.get_mut(*muscle).unwrap();
                let c1 = structure.edge_to_entity(m_data.anchor1.unwrap()).unwrap();
                let c2 = structure.edge_to_entity(m_data.anchor2.unwrap()).unwrap();

                *m_transform = get_muscle_transform(&transform_q, c1, c2,);
            }
        }
    }

    for (muscle, mut m_transform) in muscle_set.p0().iter_mut() { // update position on newly added
        if muscle.anchor1.is_none() || muscle.anchor2.is_none() {
            return
        }

        let c1 = structure.edge_to_entity(muscle.anchor1.unwrap()).unwrap();
        let c2 = structure.edge_to_entity(muscle.anchor2.unwrap()).unwrap();

        *m_transform = get_muscle_transform(&transform_q, c1, c2,);
    }
}

/// Returns the muscle transform given the two anchor connectors.
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