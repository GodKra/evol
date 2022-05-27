use bevy::{prelude::*};

use crate::{
    points::*,
};

/// System that saves the joint structure to a data file (currently ./points.ron)
/// 
/// *active
pub fn save(
    input: Res<Input<KeyCode>>,
    transform_q: Query<&GlobalTransform, With<Joint>>,
    root_q: Query<Entity, With<Root>>,
    child_q: Query<&Children, With<Joint>>,
) {
    if !input.just_pressed(KeyCode::S) {
        return;
    }
    let root = root_q.single();

    let points = Point { 
        connections: make_points(&root, &child_q, &transform_q),
        ..default()
    };

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