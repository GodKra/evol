use std::collections::HashMap;

use bevy::{prelude::*};
use bevy_mod_picking::PickingCamera;
use serde::{Serialize, Deserialize};

use crate::util::{JointMaterial, JointMeshes, Errors};

use super::{selection::EntitySelected, IsMuscleMode, joint::{Connector, IDMap, Joint}};

/// Component to each muscle describing its anchors. Anchor1 is always lower in index than anchor2 (to simplify saving).
#[derive(Component, Default, Debug)]
pub struct Muscle {
    pub anchor1: u32,
    pub anchor2: u32,
}

/// Component to each joint with muscles describing each muscle and its connected pair (joint).
#[derive(Component, Default, Debug)]
pub struct MuscleConnectors {
    pub pair: HashMap<u32, Entity>
}

/// Serialized form of all [Muscles] components combined
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MuscleData {
    pub pairs: HashMap<u32, Vec<u32>>,
}

/// Struct used to store incomplete muscles (since joints are a tree structure) when deserializing [MuscleData]
#[derive(Default, Debug)]
pub struct MuscleHalfs {
    pub pairs: HashMap<u32, Vec<(u32, Entity)>>,
}

/// (Joint, Muscle); describes the first anchor when creating a muscle.
#[derive(Default, Resource)]
pub struct MuscleRoot(Option<(Entity, Entity)>);

/// Marker for Entity used to give an anchor (cursor's projected position) to muscles which does not have a second anchor
/// **TODO: muscles should follow the cursor when anchor is set to this**
#[derive(Component)]
pub struct CursorAnchor;

/// Creates muscles between two connectors with proper anchors. Muscle creation mode is initiated with M button.
pub fn muscle_construct(
    mut commands: Commands,
    meshes: Res<JointMeshes>,
    materials: Res<JointMaterial>,
    mut is_muscle_mode: ResMut<IsMuscleMode>,
    key_input: Res<Input<KeyCode>>,
    // mouse_input: Res<Input<MouseButton>>,
    entity_selected: Res<EntitySelected>,
    mut id_map: ResMut<IDMap>,
    mut muscle_root: ResMut<MuscleRoot>,
    added_pick_cam: Query<&PickingCamera, Added<PickingCamera>>,
    connector_q: Query<&Connector>,
    mut muscle_con_q: Query<&mut MuscleConnectors>,
    // anchor_q: Query<&CursorAnchor>,
    mut muscle_q: Query<&mut Muscle>,
) {
    for _ in added_pick_cam.iter() { // runs only once when initializing.
        let anchor = commands.spawn((TransformBundle::default(), CursorAnchor)).id();

        id_map.0.insert(0, anchor);
    }

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
    let joint = connector.head_joint;
    if muscle_root.0.is_none() { // No anchor set yet
        let Some(joint_id) = id_map.0.get_by_right(&joint) else {
            panic!("{}", Errors::IDMapIncompleteError(None, Some(joint)))
        };
        let muscle = commands.spawn((
            PbrBundle {
                mesh: meshes.connector.clone(),
                material: materials.muscle_color.clone(),
                ..Default::default()
            },
            Muscle { anchor1: *joint_id, anchor2: 0 }
        )).id();
        muscle_root.0 = Some((joint, muscle));
    } else {
        if muscle_root.0.unwrap().0 == joint { // Root joint is selected again as second anchor
            return;
        }
        let Some(joint_id) = id_map.0.get_by_right(&joint) else {
            panic!("{}", Errors::IDMapIncompleteError(None, Some(joint)))
        };

        let (anchor1, muscle) = muscle_root.0.unwrap();
        let mut muscle_comp = muscle_q.get_mut(muscle).unwrap();
        
        muscle_comp.anchor2 = *joint_id.max(&muscle_comp.anchor1);
        muscle_comp.anchor1 = *joint_id.min(&muscle_comp.anchor1);

        // Anchor1
        let Ok(mut muscle_con) = muscle_con_q.get_mut(anchor1) else {
            panic!("{}", Errors::ComponentMissingError("MuscleConnectors", anchor1))
        };
        if muscle_con.pair.contains_key(joint_id) {
            println!(":: Muscle already exists");
            is_muscle_mode.0 = false;
            commands.entity(muscle).despawn();
            muscle_root.0 = None;
            return;
        }
        muscle_con.pair.insert(*joint_id, muscle);

        // Anchor2
        let Some(anchor1_id) = id_map.0.get_by_right(&anchor1) else {
            panic!("{}", Errors::IDMapIncompleteError(None, Some(anchor1)))
        };
        let Ok(mut muscle_con) = muscle_con_q.get_mut(joint) else {
            panic!("{}", Errors::ComponentMissingError("MuscleConnectors", joint))
        };
        muscle_con.pair.insert(*anchor1_id, muscle);

        is_muscle_mode.0 = false;
        muscle_root.0 = None;
        println!(":: Muscle Constructed: {:?} <> {:?}", joint_id, anchor1_id);
    }
}

/// Updates the transform of the muscles when they have been created or the anchors have been moved.
pub fn update_muscles(
    id_map: Res<IDMap>,
    mut muscle_set: ParamSet<(
        Query<(&Muscle, &mut Transform), Changed<Muscle>>,
        Query<&mut Transform>,
    )>,
    changed_connectors: Query<(Entity, &MuscleConnectors), Changed<GlobalTransform>>,
    joint_q: Query<(&Joint, &GlobalTransform)>,
) {
    for (j1, connectors) in changed_connectors.iter() { // update position on joint movement
        for (j2, muscle) in connectors.pair.iter() {
            let Some(j2) = id_map.0.get_by_left(j2) else {
                panic!("{}", Errors::IDMapIncompleteError(Some(*j2), None))
            };

            let mut transform_q = muscle_set.p1();
            let mut m_transform = transform_q.get_mut(*muscle).unwrap();
            *m_transform = get_muscle_transform(j1, *j2, &joint_q);
        }
    }

    for (muscle, mut m_transform) in muscle_set.p0().iter_mut() { // update position on newly added
        if muscle.anchor1 == 0 || muscle.anchor2 == 0 {
            return
        }
        let Some(j1) = id_map.0.get_by_left(&muscle.anchor1) else {
            panic!("{}", Errors::IDMapIncompleteError(Some(muscle.anchor1), None))
        };
        let Some(j2) = id_map.0.get_by_left(&muscle.anchor2) else {
            panic!("{}", Errors::IDMapIncompleteError(Some(muscle.anchor2), None))
        };

        *m_transform = get_muscle_transform(*j1, *j2, &joint_q,);
    }
}

/// Returns the muscle transform given the two anchor joints.
fn get_muscle_transform(
    joint1: Entity,
    joint2: Entity,
    joint_transforms: &Query<(&Joint, &GlobalTransform)>,
) -> Transform {
    let (j1, j1_transform) = joint_transforms.get(joint1).unwrap();
    let (j2, j2_transform) = joint_transforms.get(joint2).unwrap();
    let (_, j1_parent) = joint_transforms.get(j1.parent.unwrap()).unwrap();
    let (_, j2_parent) = joint_transforms.get(j2.parent.unwrap()).unwrap();

    let j1_mid = (j1_transform.translation()+j1_parent.translation())/Vec3::new(2.0, 2.0, 2.0);
    let j2_mid = (j2_transform.translation()+j2_parent.translation())/Vec3::new(2.0, 2.0, 2.0);

    let translation = (j1_mid+j2_mid)/Vec3::new(2.0, 2.0, 2.0);
    let scale = Vec3::new(0.5, j1_mid.distance(j2_mid)/2.0, 0.5) ;
    let rotation = Quat::from_rotation_arc(Vec3::Y, (j1_mid-translation).normalize());
    Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, translation))
}