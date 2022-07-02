use bevy::{prelude::*};

use super::{selection::JointSelected, joint::*};
/// System that saves the joint structure to a data file (currently ./points.ron)
/// 
/// *active
pub fn save(
    transform_q: Query<&GlobalTransform, With<Joint>>,
    root_q: Query<Entity, With<Root>>,
    child_q: Query<&Children, With<Joint>>,
) {
    let root = root_q.single();

    let points = Point { 
        connections: make_points(&root, &child_q, &transform_q),
        ..default()
    };
    println!("saved");

    std::fs::write(
        "./points.ron", 
        ron::ser::to_string_pretty(
            &points, 
            ron::ser::PrettyConfig::new()
                .indentor(" ".to_string())
        ).unwrap()
    ).unwrap();
}

fn make_points(
    parent: &Entity, 
    child_q: &Query<&Children, With<Joint>>, 
    transform_q: &Query<&GlobalTransform, With<Joint>>,
) -> Vec<Point> {
    let mut p = Vec::new();

    let children = child_q.get(*parent);
    if children.is_err() {
        return Vec::new();
    }
    for joint in children.unwrap().iter() {
        let mut point = Point::default();
        let p_transform = transform_q.get(*parent).unwrap(); // always a joint
        let s_transform = match transform_q.get(*joint) { // ignore rotator
            Ok(t) => t,
            Err(_) => continue,
        };
        let dir = s_transform.translation - p_transform.translation;
        let dir = (dir * 1000.0).round() / 1000.0; // 3 d.p to make it cleaner
        point.r_coords = (dir.x, dir.y, dir.z);
        point.connections = make_points(joint, child_q, transform_q);
        p.push(point);
    }
    p
}

/// System to handle deletion of joints
/// 
/// *active
pub fn delete_joint(
    mut commands: Commands,
    mut joint_selected: ResMut<JointSelected>,
    joint_q: Query<&Joint>,
) {
    if joint_selected.0.is_none() {
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