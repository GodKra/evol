use bevy::{prelude::*, input::{mouse::MouseMotion}};
use crate::{
    util::*,
};
use super::{*, pgraph::*};

/// System to handle the movement of the joints when in Grab/Rotate edit modes.
/// 
/// *passive
pub fn adjust_control(
    pgraph: Res<PGraph>,
    mouse_input: Res<Input<MouseButton>>,
    key_input: Res<Input<KeyCode>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut windows: ResMut<Windows>,
    is_adjust_mode: Res<IsAdjustMode>,
    entity_selected: Res<EntitySelected>,
    pos_cache: Res<PositionCache>,
    mut mv_cache: ResMut<MovementCache>,
    cam_query: Query<(&Camera, &GlobalTransform)>,
    joint_query: Query<&Joint>,
    mut editable_query: Query<&mut Editable>,
    mut transform_query: Query<&mut Transform>,
    // global_query: Query<&mut GlobalTransform, Without<Camera>>,
) {
    if !is_adjust_mode.0 || !entity_selected.is_joint() {
        return;
    } else if mouse_input.just_pressed(MouseButton::Left) || key_input.just_pressed(KeyCode::Escape) {
        let Some(window) = windows.get_primary_mut() else {
            panic!("{}", Errors::Window);
        };
        window.set_cursor_grab_mode(bevy::window::CursorGrabMode::None);
        window.set_cursor_visibility(true);
        return;
    }

    let joint = entity_selected.get().unwrap();
    let Ok(mut editable) = editable_query.get_mut(joint) else {
        panic!("{}", Errors::ComponentMissing("Editable", joint));
    };
    let (cam, cam_transform) = cam_query.single();

    let mut mouse_move = Vec2::ZERO;
    
    for ev in motion_evr.iter() {
        mouse_move += ev.delta;
    }
    let editable_mode = if editable.mode.is_some() {
        editable.mode.as_ref().unwrap()
    } else {
        println!("** Editable mode missing for entity {:?}", joint);
        return;
    };

    match editable_mode {
        EditMode::GrabExtend => {
            let Ok(point) = joint_query.get(joint) else {
                panic!("{}", Errors::ComponentMissing("Joint", joint));
            };
            
            if pgraph.node_parent(point.node_index).is_none() {
                editable.mode = Some(EditMode::GrabFull);
                return;
            }

            let parent = pgraph.node_parent_entity(point.node_index).unwrap();

            let p_translation = transform_query.get(parent).unwrap().translation;
            let mut j_transform = transform_query.get_mut(joint).unwrap();
            let p_pos = cam.world_to_viewport(cam_transform, p_translation);
            let s_pos = cam.world_to_viewport(cam_transform, j_transform.translation);
            if p_pos.is_none() || s_pos.is_none() {
                return;
            }

            let dir_vec = s_pos.unwrap()-p_pos.unwrap();
            let mouse_move = Vec2::new(mouse_move.x, -mouse_move.y); // mouse_move.y seems to be inverted
            let mv = mouse_move.dot(dir_vec.normalize());

            let relative_pos = j_transform.translation-p_translation;

            let mut pos = relative_pos + ((mv * 0.01) * relative_pos.normalize());
            mv_cache.0 += mv * 0.01;
            
            if key_input.pressed(KeyCode::LControl) {
                pos = (mv_cache.0/2.0).round()*2.0 * relative_pos.normalize();
            }

            // prevents extension to negative
            if relative_pos.dot(pos - 2.0 * relative_pos.normalize()) < 0.0  {
                pos = 2.0 * relative_pos.normalize();
                mv_cache.0 = 2.0;
            }

            j_transform.translation = pos + p_translation;

            let Some(window) = windows.get_primary_mut() else {
                panic!("{}", Errors::Window);
            };
            window.set_cursor_grab_mode(bevy::window::CursorGrabMode::Locked);
            window.set_cursor_visibility(false);
            window.set_cursor_position(s_pos.unwrap());
        },
        // both has similar logic
        EditMode::RotateFull | EditMode::GrabFull => {
            let Some(window) = windows.get_primary() else {
                panic!("{}", Errors::Window);
            };

            let Some(mouse_pos) = window.cursor_position() else {
                return
            };
            
            let ray = cam.viewport_to_world(cam_transform, mouse_pos).unwrap();

            let point = joint_query.get(joint).unwrap();

            let parent = pgraph.node_parent_entity(point.node_index).unwrap_or(joint);

            let p_translation = transform_query.get(parent).unwrap().translation;
            let mut j_transform = transform_query.get_mut(joint).unwrap();
            
            // https://antongerdelan.net/opengl/raycasting.html
            let radius = (j_transform.translation-p_translation).length();
            let rot_to_ray = ray.origin - p_translation;
            let joint_to_ray = ray.origin - j_transform.translation;
            let b = ray.direction.dot(rot_to_ray);
            let c = rot_to_ray.dot(rot_to_ray) - radius * radius;
            // rotation is spherical when within radius
            let len = if b*b - c >= 0.0 && editable.mode.as_ref().unwrap() != &EditMode::GrabFull {
                let root = (b*b-c).sqrt();
                if joint_to_ray.dot(rot_to_ray.normalize()) < rot_to_ray.length() {
                    // joint is facing camera
                    (-b + root).min(-b - root)
                } else {
                    // joint is facing other side
                    (-b + root).max(-b - root)
                }
            // planar rotation outside radius ** TODO: fix slow movement when transitioning **
            } else {
                let a = cam_transform.forward();
                let b = j_transform.translation-ray.origin;
                let c = ray.direction;
                // length of c at which b - c is orthogonal to a, just (Cos0 = A / H) in vectors
                (a.dot(b)/a.length()) / (a.dot(c)/(a.length()*c.length()))
            };

            let ray_pos = ray.direction * len;
            let mouse_pos = ray.origin + ray_pos;
            if let Some(EditMode::GrabFull) = editable.mode {
                j_transform.translation = mouse_pos;
            } else {
                let dir_vec = (mouse_pos-p_translation).normalize();
                j_transform.translation = p_translation + (radius * dir_vec);
            }
        },
        EditMode::GrabAxis(axis) => {
            let Ok(point) = joint_query.get(joint) else {
                panic!("{}", Errors::ComponentMissing("Joint", joint));
            };
            
            if pgraph.node_parent(point.node_index).is_none() {
                editable.mode = Some(EditMode::GrabFull);
                return;
            }

            let j_transform = transform_query.get(joint).unwrap().translation;
            let p_pos = cam.world_to_viewport(cam_transform, j_transform + axis.to_vec());
            let s_pos = cam.world_to_viewport(cam_transform, j_transform);
            if p_pos.is_none() || s_pos.is_none() {
                return;
            }

            let dir_vec = s_pos.unwrap()-p_pos.unwrap();
            let mouse_move = Vec2::new(-mouse_move.x, mouse_move.y); // mouse_move.x seems to be inverted
            let mv = mouse_move.dot(dir_vec.normalize());
            mv_cache.0 += mv * 0.01;

            let mut transform = transform_query.get_mut(joint).unwrap();

            let mut pos = transform.translation + (mv * 0.02) * axis.to_vec();

            if key_input.pressed(KeyCode::LControl) {
                pos = pos_cache.0 + (mv_cache.0/2.0).round()*2.0 * axis.to_vec();
            }

            transform.translation = pos;

            let Some(window) = windows.get_primary_mut() else {
                panic!("{}", Errors::Window);
            };
            window.set_cursor_grab_mode(bevy::window::CursorGrabMode::Locked);
            window.set_cursor_visibility(false);
            window.set_cursor_position(s_pos.unwrap());
        },
        EditMode::RotateAxis(axis) => {
            let Some(window) = windows.get_primary() else {
                panic!("{}", Errors::Window);
            };

            let Some(mouse_pos) = window.cursor_position() else {
                return
            };
            
            let ray = cam.viewport_to_world(cam_transform, mouse_pos).unwrap();

            let Ok(point) = joint_query.get(joint) else {
                panic!("{}", Errors::ComponentMissing("Joint", joint));
            };
            
            if pgraph.node_parent(point.node_index).is_none() {
                editable.mode = Some(EditMode::GrabFull);
                return;
            }
    
            let parent = pgraph.node_parent_entity(point.node_index).unwrap();
            
            let p = transform_query.get(parent).unwrap().translation;
            
            let mut j_transform = transform_query.get_mut(joint).unwrap();
            let j = j_transform.translation;

            let relative_pos = j-p;
            
            if relative_pos.cross(axis.to_vec()) == Vec3::ZERO {
                // j_transform.translation = pos_cache.0;
                return;
            }

            let center = p + axis.to_vec() * relative_pos.dot(axis.to_vec());
            let intersection = get_intersect_plane_ray(center, axis.to_vec(), ray);
            let dir_vec = intersection - center;


            let rot = get_axis_rotation(j-center, dir_vec, axis.to_vec());

            j_transform.translation = p + rot * relative_pos;
        },
        _ => (),
    }
}

pub fn update_pgraph_pos(
    mut pgraph: ResMut<PGraph>,
    changed_q: Query<(&Joint, &Transform), Changed<Transform>>,
) {
    for (joint, transform) in changed_q.iter() {
        let weight = pgraph.0.node_weight_mut(joint.node_index).unwrap();
        weight.pos = transform.translation;
    }
}

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