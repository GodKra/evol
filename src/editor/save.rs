use bevy::{prelude::*, utils::HashMap};

use crate::editor::body::Body;


use super::{selection::EntitySelected, joint::*, muscle::{MuscleData, Muscle, MuscleConnectors}};


/// A map used to fix the out-of-order IDs (caused when adding and removing muscles) in the ID Map when saving.
#[derive(Default, Debug)]
struct ReplaceMap(HashMap<u32, u32>);

/// System that saves the joint structure to a data file (currently ./points.ron)
/// 
/// *active
pub fn save(
    id_map: Res<IDMap>,
    transform_q: Query<&GlobalTransform, With<Joint>>,
    root_q: Query<Entity, With<Root>>,
    child_q: Query<&Children, With<Joint>>,
    joint_q: Query<&Joint>,
    muscle_q: Query<&Muscle>,
    muscle_con_q: Query<&MuscleConnectors>,
) {
    let root = root_q.single();
    let mut replace_map = ReplaceMap::default();
    // let mut muscle_data = MuscleData::default();

    let points = Point { 
        connections: make_points(
            &mut 1,
            &root, 
            &mut replace_map,
            &id_map,
            &child_q, 
            &transform_q,
            &joint_q,
            &muscle_con_q,
        ),
        ..default()
    };


    let body = Body {
        points,
        muscle: make_muscles(&replace_map, &muscle_q)
    };

    std::fs::write(
        "./points.ron", 
        ron::ser::to_string_pretty(
            &body, 
            ron::ser::PrettyConfig::new()
                .indentor(" ".to_string())
        ).unwrap()
    ).unwrap();

    println!("** SAVED");
}

fn make_points(
    index: &mut u32,
    parent: &Entity,
    replace_map: &mut ReplaceMap,
    id_map: &Res<IDMap>,
    child_q: &Query<&Children, With<Joint>>, // With<> restriction doesn't work with children?
    transform_q: &Query<&GlobalTransform, With<Joint>>,
    joint_q: &Query<&Joint>,
    muscle_con_q: &Query<&MuscleConnectors>,
) -> Vec<Point> {
    let mut p = Vec::new();

    let children = child_q.get(*parent);
    if children.is_err() {
        return Vec::new();
    }
    for joint in children.unwrap().iter() {
        if !joint_q.contains(*joint) { // With<> resitriction problem
            continue;
        }
        *index = *index + 1;

        let stored_id = id_map.0.get_by_right(joint).unwrap();
        replace_map.0.insert(*stored_id, *index);

        let mut point = Point::default();
        let p_transform = transform_q.get(*parent).unwrap().translation(); // always a joint
        let s_transform = match transform_q.get(*joint) { // ignore rotator
            Ok(t) => t.translation(),
            Err(_) => continue,
        };
        let dir = s_transform - p_transform;
        let dir = (dir * 1000.0).round() / 1000.0; // 3 d.p to make it cleaner
        point.r_coords = (dir.x, dir.y, dir.z);
        point.connections = make_points(
            index, 
            joint, 
            replace_map, 
            id_map,
            child_q, 
            transform_q, 
            joint_q, 
            muscle_con_q, 
        );
        p.push(point);
    }
    p
}

/// creates MuscleData from all the [Muscle]s in the scene. 
fn make_muscles(
    replace_map: &ReplaceMap,
    muscle_q: &Query<&Muscle>,
) -> MuscleData {
    let mut muscle_vec = Vec::new();
    for muscle in muscle_q {
        let a1 = replace_map.0.get(&muscle.anchor1).unwrap();
        let a2 = replace_map.0.get(&muscle.anchor2).unwrap();
        muscle_vec.push((*a1.min(a2), *a1.max(a2))); // ordering here not necessary but keeping to make sure
    }

    let mut m = MuscleData::default();
    for (k, v) in muscle_vec {
        m.pairs.entry(k).or_insert_with(Vec::new).push(v)
    }
    m
}

/// System to handle deletion of joints
/// 
/// *active
pub fn delete_joint(
    mut commands: Commands,
    mut entity_selected: ResMut<EntitySelected>,
    id_map: Res<IDMap>,
    joint_q: Query<&Joint>,
    mut muscle_con_q: Query<&mut MuscleConnectors>,
) {
    if !entity_selected.is_joint() {
        return;
    }

    let joint = entity_selected.get().unwrap();
    let joint_info = joint_q.get(joint).unwrap();
    let muscle_info = muscle_con_q.get(joint).unwrap();

    for (id, muscle) in muscle_info.pair.clone().iter() { // clone workaround for borrowchecker
        commands.entity(*muscle).despawn();
        let pair_joint = id_map.0.get_by_left(id).unwrap();
        let mut pair_info = muscle_con_q.get_mut(*pair_joint).unwrap();
        pair_info.pair.remove(id_map.0.get_by_right(&joint).unwrap());
        
    }

    if let Some(rotator) = joint_info.rotator {
        commands.entity(rotator).despawn();
    }
    if let Some(connector) = joint_info.connector {
        commands.entity(connector).despawn();
    }
    
    commands.entity(joint).despawn_recursive();
    entity_selected.set(None);
}