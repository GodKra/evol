
use bevy::{prelude::*};
use bevy_mod_picking::{PickableMesh};
use petgraph::stable_graph::EdgeIndex;

use crate::util::{JointMaterial, JointMeshes};

use super::{selection::*, IsMuscleMode, pgraph::*};

/// Component to each muscle describing its anchors. Anchor1 is always lower in index than anchor2 (to simplify saving).
#[derive(Component, Default, Debug)]
pub struct Muscle {
    pub anchor1: Option<EdgeIndex>,
    pub anchor2: Option<EdgeIndex>,
}

/// (Joint, Muscle); describes the first anchor when creating a muscle.
#[derive(Default, Resource)]
pub struct MuscleRoot(Option<(EdgeIndex, Entity)>);

/// Marker for Entity used to give an anchor (cursor's projected position) to muscles which does not have a second anchor
/// **TODO: muscles should follow the cursor when anchor is set to this**
#[derive(Component)]
pub struct CursorAnchor;

/// Creates muscles between two connectors with proper anchors. Muscle creation mode is initiated with M button.
pub fn muscle_construct(
    mut pgraph: ResMut<PGraph>,
    mut commands: Commands,
    meshes: Res<JointMeshes>,
    materials: Res<JointMaterial>,
    mut is_muscle_mode: ResMut<IsMuscleMode>,
    key_input: Res<Input<KeyCode>>,
    // mouse_input: Res<Input<MouseButton>>,
    entity_selected: Res<EntitySelected>,
    mut muscle_root: ResMut<MuscleRoot>,
    // added_pick_cam: Query<&PickingCamera, Added<PickingCamera>>,
    connector_q: Query<&Connector>,
    // anchor_q: Query<&CursorAnchor>,
    mut muscle_q: Query<&mut Muscle>,
) {
    // for _ in added_pick_cam.iter() { // runs only once when initializing.
    //     let anchor = commands.spawn((TransformBundle::default(), CursorAnchor)).id();
    // }

    if !is_muscle_mode.0 && entity_selected.is_connector() && key_input.just_pressed(KeyCode::M) { // First anchor is set
        is_muscle_mode.0 = true;
        return;
    }
    if !is_muscle_mode.0 || !entity_selected.is_connector() { // Entity selected is not a muscle anchor so reset
        if is_muscle_mode.0 {
            is_muscle_mode.0 = false;
            if let Some((_, muscle)) = muscle_root.0 {
                commands.entity(muscle).despawn();
            }
            muscle_root.0 = None;
        }
        return;
    }

    let connector = connector_q.get(entity_selected.get().unwrap()).unwrap(); // both unwraps are certain to work with earlier checks
    
    if muscle_root.0.is_none() { // No anchor set yet\

        let muscle = commands.spawn((
            PbrBundle {
                mesh: meshes.connector.clone(),
                material: materials.muscle_color.clone(),
                ..Default::default()
            },
            Muscle { anchor1: Some(connector.edge_index), anchor2: None },
            PickableMesh::default(),
        )).id();
        commands.entity(muscle).insert(Selectable::with_type(SelectableEntity::Muscle(muscle)));
        muscle_root.0 = Some((connector.edge_index, muscle));
    } else {
        if muscle_root.0.unwrap().0 == connector.edge_index { // Root joint is selected again as second anchor
            return;
        }

        let (anchor1, muscle) = muscle_root.0.unwrap();
        let anchor2 = connector.edge_index;
        let mut muscles = muscle_q.get_mut(muscle).unwrap();
        
        muscles.anchor2 = Some(connector.edge_index);

        // Anchor1
        let anchor1_data = pgraph.0.edge_weight_mut(anchor1).unwrap();
        if anchor1_data.muscles.contains_key(&anchor2) {
            println!(":: Muscle already exists");
            is_muscle_mode.0 = false;
            commands.entity(muscle).despawn();
            muscle_root.0 = None;
            return;
        }
        anchor1_data.muscles.insert(anchor2, muscle);

        // Anchor2
        let anchor2_data = pgraph.0.edge_weight_mut(anchor2).unwrap();
        anchor2_data.muscles.insert(anchor1, muscle);

        is_muscle_mode.0 = false;
        muscle_root.0 = None;
        println!(":: Muscle Constructed: {:?} <> {:?}", anchor1, anchor2);
    }
}

/// Updates the transform of the muscles when they have been created or the anchors have been moved.
pub fn update_muscles(
    pgraph: Res<PGraph>,
    mut muscle_set: ParamSet<(
        Query<(&Muscle, &mut Transform), (Changed<Muscle>, Without<Joint>)>,
        Query<(&Muscle, &mut Transform), (With<Muscle>, Without<Joint>)>,
    )>,
    changed_joints: Query<&Joint, Changed<Transform>>,
    transform_q: Query<&Transform, (Without<Muscle>, Without<Joint>)>,
) {
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

    for (muscle, mut m_transform) in muscle_set.p0().iter_mut() { // update position on newly added
        if muscle.anchor1 == None || muscle.anchor2 == None {
            return
        }

        let c1 = pgraph.edge_to_entity(muscle.anchor1.unwrap()).unwrap();
        let c2 = pgraph.edge_to_entity(muscle.anchor2.unwrap()).unwrap();

        *m_transform = get_muscle_transform(&transform_q, c1, c2,);
    }
}

/// Returns the muscle transform given the two anchor joints.
fn get_muscle_transform(
    transform_q: &Query<&Transform, (Without<Muscle>, Without<Joint>)>,
    connector1: Entity,
    connector2: Entity,
    // joint_transforms: &Query<(&Joint, &GlobalTransform)>,
) -> Transform {
    let c1 = transform_q.get(connector1).unwrap().translation;
    let c2 = transform_q.get(connector2).unwrap().translation;

    let translation = (c1+c2) / 2.0;
    let scale = Vec3::new(0.5, c1.distance(c2) / 2.0, 0.5) ;
    let rotation = Quat::from_rotation_arc(Vec3::Y, (c1-translation).normalize());
    Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, translation))
}