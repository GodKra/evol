use bevy::prelude::*;
use bevy_rapier3d::prelude::ExternalForce;

use petgraph::graph::*;

use crate::{pgraph::*, selection::EntitySelected};

#[derive(Debug, Clone, Default)]
pub enum MuscleActivity {
    #[default]
    Relax, 
    Contract,
    // Extend,
    ReboundContract,
    // RebountExtend,
}

#[derive(Default, Debug, Clone, Component)]
pub struct MuscleState(pub MuscleActivity);

#[derive(Default, Clone, Component)]
pub struct MuscleLength {
    starting: f32,
    current: f32,
    min: f32,
}

/// Updates the transform of the muscles when they have been created or the anchors have been moved.
pub fn update_muscles(
    pgraph: Res<PGraph>,
    mut muscle_set: ParamSet<(
        Query<(Entity, &Muscle, &mut Transform), (Added<Muscle>, Without<Joint>)>,
        Query<(Entity, &Muscle, &mut Transform), (With<Muscle>, Without<Joint>)>,
    )>,
    changed_joints: Query<&Joint, Changed<Transform>>,
    transform_q: Query<&Transform, (Without<Muscle>, Without<Joint>)>,
    mut length_q: Query<&mut MuscleLength>,
) {
    for (id, muscle, mut m_transform) in muscle_set.p0().iter_mut() { // update position on start 
        if muscle.anchor1 == None || muscle.anchor2 == None {
            return
        }

        let con1 = pgraph.edge_to_entity(muscle.anchor1.unwrap()).unwrap();
        let con2 = pgraph.edge_to_entity(muscle.anchor2.unwrap()).unwrap();

        
        let c1 = transform_q.get(con1).unwrap();
        let c2 = transform_q.get(con2).unwrap();
        
        let c1_len = c1.scale.y * 2.0;
        let c2_len = c2.scale.y * 2.0;

        let mut muscle_len = length_q.get_mut(id).unwrap();
        
        (*m_transform, muscle_len.starting) = get_muscle_transform_len(c1.translation, c2.translation);
        muscle_len.current = muscle_len.starting;
        muscle_len.min = (c1_len-c2_len).abs() / 2.0;

    }

    for joint in changed_joints.iter() {
        let edges = pgraph.0.edges(joint.node_index);
        for edge in edges {
            let e_weight = edge.weight();
            for (_, muscle) in e_weight.muscles.iter() {
                let mut query = muscle_set.p1();
                let (id, m_data, mut m_transform) = query.get_mut(*muscle).unwrap();
                let con1 = pgraph.edge_to_entity(m_data.anchor1.unwrap()).unwrap();
                let con2 = pgraph.edge_to_entity(m_data.anchor2.unwrap()).unwrap();

                let c1 = transform_q.get(con1).unwrap();
                let c2 = transform_q.get(con2).unwrap();

                let mut m_len = length_q.get_mut(id).unwrap();
        
                (*m_transform, m_len.current) = get_muscle_transform_len(c1.translation, c2.translation);
            }
        }
    }
}

/// Returns the muscle transform and length given the two anchor joints.
fn get_muscle_transform_len(
    connector1: Vec3,
    connector2: Vec3,
) -> (Transform, f32) {

    let len = connector1.distance(connector2);

    let translation = (connector1+connector2) / 2.0;
    let scale = Vec3::new(0.5, len / 2.0, 0.5) ;
    let rotation = Quat::from_rotation_arc(Vec3::Y, (connector1-translation).normalize());
    (Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, translation)), len)
}

pub fn activate_muscle(
    pgraph: Res<PGraph>,
    mut muscle_q: Query<(&Muscle, &MuscleLength, &mut MuscleState, &Transform)>,
    transform_q: Query<&Transform, Without<Muscle>>,
    mut force_q: Query<&mut ExternalForce>,

) {
    let force_coefficient = 100.0;

    for (muscle, len, mut state, transform) in muscle_q.iter_mut() {
        match state.0 {
            MuscleActivity::Relax => {

            },
            MuscleActivity::Contract => {
                if len.current < len.min + 1.0 {
                    state.0 = MuscleActivity::ReboundContract;
                    break;
                }
                println!("CONTRACT {}, {}", len.min, len.current);
                
                let (c1, c2) = get_connector_dir(&pgraph, muscle, transform.translation, &transform_q);

                add_force_to_endpoints(&pgraph, muscle.anchor1.unwrap(), c1 * force_coefficient, &mut force_q);
                add_force_to_endpoints(&pgraph, muscle.anchor2.unwrap(), c2 * force_coefficient, &mut force_q);
            },
            // MuscleActivity::Extend => {

            // },
            MuscleActivity::ReboundContract => {
                if len.current > len.min + 1.0 {
                    state.0 = MuscleActivity::Contract;
                    break;
                }

                println!("REBOUND {}, {}", len.min, len.current);
                
                let (c1, c2) = get_connector_dir(&pgraph, muscle, transform.translation, &transform_q);

                add_force_to_endpoints(&pgraph, muscle.anchor1.unwrap(), -c1 * force_coefficient, &mut force_q);
                add_force_to_endpoints(&pgraph, muscle.anchor2.unwrap(), -c2 * force_coefficient, &mut force_q);
            },
            // MuscleActivity::RebountExtend => {

            // },
        }
    }
}

pub fn contract_muscle (
    key_input: Res<Input<KeyCode>>,
    entity_selected: Res<EntitySelected>,
    mut muscle_q: Query<&mut MuscleState>,
) {
    if !key_input.just_pressed(KeyCode::C) || !entity_selected.is_muscle() {
        return;
    }
    println!("contracting");

    let selected = entity_selected.get().unwrap();
    let mut muscle = muscle_q.get_mut(selected).unwrap();
    muscle.0 = MuscleActivity::Contract;
}

fn add_force_to_endpoints(
    pgraph: &Res<PGraph>, 
    edge: EdgeIndex,
    force: Vec3,
    force_q: &mut Query<&mut ExternalForce>
) {
    let (j1, j2) = pgraph.0.edge_endpoints(edge).unwrap();
    let joint1 = pgraph.node_to_entity(j1).unwrap();
    let joint2 = pgraph.node_to_entity(j2).unwrap();
    let mut j1_force = force_q.get_mut(joint1).unwrap();
    j1_force.force += force;
    let mut j2_force = force_q.get_mut(joint2).unwrap();
    j2_force.force += force;
}

fn get_connector_dir(
    pgraph: &Res<PGraph>,
    muscle: &Muscle,
    muscle_pos: Vec3,
    transform_q: &Query<&Transform, Without<Muscle>>,
) -> (Vec3, Vec3) {
    let c1 = pgraph.edge_to_entity(muscle.anchor1.unwrap()).unwrap();
    let c2 = pgraph.edge_to_entity(muscle.anchor2.unwrap()).unwrap();

    let c1_pos = transform_q.get(c1).unwrap().translation;
    let c2_pos = transform_q.get(c2).unwrap().translation;

    ((muscle_pos-c1_pos).normalize(), (muscle_pos-c2_pos).normalize())
}

pub fn clear_forces(
    mut force_q: Query<&mut ExternalForce>,
) {
    for mut force in force_q.iter_mut() {
        force.force = Vec3::ZERO;
    }
}