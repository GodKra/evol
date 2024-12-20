use bevy::{prelude::*, input::mouse::MouseMotion, window::PrimaryWindow};

use super::controls::{ActionEvent, CursorControlEvent, EditMode};
use crate::{structure::*, util::*};

/// System to handle the movement of the joints when in adjust modes.
pub fn adjust_control(
    mut commands: Commands,
    structure: Res<Structure>,
    edit_mode: Res<EditMode>,
    mut motion_evr: EventReader<MouseMotion>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    cam_query: Query<(&Camera, &GlobalTransform)>,
    joint_query: Query<&Joint>,
    mut transform_query: Query<&mut Transform>,
) {
    if matches!(*edit_mode, EditMode::Default) {
        return;
    }

    let (cam, cam_transform) = cam_query.single();

    let mut mouse_move = Vec2::ZERO;
    
    for ev in motion_evr.read() {
        mouse_move += ev.delta;
    }

    match *edit_mode {
        EditMode::AdjustExtend(joint) => {
            let Ok(point) = joint_query.get(joint) else {
                panic!("{}", Errors::ComponentMissing("Joint", joint));
            };
            
            if structure.node_parent(point.node_index).is_none() {
                // Transfer to regular AdjustGrab since no parent.
                commands.send_event(ActionEvent::Cancel);
                commands.send_event(ActionEvent::AdjustGrab);
                return;
            }

            let parent = structure.node_parent_entity(point.node_index).unwrap();

            let p_translation = transform_query.get(parent).unwrap().translation;
            let mut j_transform = transform_query.get_mut(joint).unwrap();

            let p_pos = cam.world_to_viewport(cam_transform, p_translation);
            let j_pos = cam.world_to_viewport(cam_transform, j_transform.translation);

            if p_pos.is_err() || j_pos.is_err() {
                return;
            }

            let dir_vec = j_pos.unwrap()-p_pos.unwrap();
            let mv = mouse_move.dot(dir_vec.normalize());

            let relative_pos = j_transform.translation-p_translation;

            let mut pos = relative_pos + ((mv * 0.01) * relative_pos.normalize());

            // prevents extension to negative
            if relative_pos.dot(pos - (2.0 * JOINT_RADIUS) * relative_pos.normalize()) < 0.0  {
                pos = (2.0 * JOINT_RADIUS) * relative_pos.normalize();
            }

            j_transform.translation = pos + p_translation;

            commands.send_event(CursorControlEvent::Position(j_pos.unwrap()));
        },
        // both has similar logic
        EditMode::AdjustRotate(joint) | EditMode::AdjustGrab(joint) => {
            let Ok(window) = window_q.get_single() else {
                panic!("window not found");
            };

            let Some(mouse_pos) = window.cursor_position() else {
                return
            };
            
            let ray = cam.viewport_to_world(cam_transform, mouse_pos).unwrap();

            let point = joint_query.get(joint).unwrap();

            if structure.node_parent(point.node_index).is_none() {
                // Transfer to regular AdjustGrab since no parent.
                commands.send_event(ActionEvent::Cancel);
                commands.send_event(ActionEvent::AdjustGrab);
                return;
            }

            let parent = structure.node_parent_entity(point.node_index).unwrap();

            let p_translation = transform_query.get(parent).unwrap().translation;
            let mut j_transform = transform_query.get_mut(joint).unwrap();
            
            // Source: https://antongerdelan.net/opengl/raycasting.html
            let radius = (j_transform.translation-p_translation).length();
            let rot_to_ray = ray.origin - p_translation;
            let joint_to_ray = ray.origin - j_transform.translation;
            let b = ray.direction.dot(rot_to_ray);
            let c = rot_to_ray.dot(rot_to_ray) - radius * radius;

            // rotation is spherical when within radius
            let len = if b*b - c >= 0.0 && !matches!(*edit_mode, EditMode::AdjustGrab(_)) {
                let root = (b*b-c).sqrt();
                if joint_to_ray.dot(rot_to_ray.normalize()) < rot_to_ray.length() {
                    // joint is facing camera
                    (-b + root).min(-b - root)
                } else {
                    // joint is facing away from camera
                    (-b + root).max(-b - root)
                }
            // planar rotation outside radius ** TODO: fix slow movement when transitioning **
            } else {
                let a = cam_transform.forward();
                let b = j_transform.translation-ray.origin;
                let c = Vec3::from(ray.direction);
                // length of c at which b - c is orthogonal to a | (Cos0 = A / H) in vectors.
                (a.dot(b)/a.length()) / (a.dot(c)/(a.length()*c.length()))
            };

            let ray_pos = ray.direction * len;
            let mouse_pos = ray.origin + ray_pos;

            if matches!(*edit_mode, EditMode::AdjustGrab(_)) {
                j_transform.translation = mouse_pos;
            } else {
                let dir_vec = (mouse_pos-p_translation).normalize();
                j_transform.translation = p_translation + (radius * dir_vec);
            }
        },
        EditMode::AdjustAxis(joint, axis) => {
            let Ok(point) = joint_query.get(joint) else {
                panic!("{}", Errors::ComponentMissing("Joint", joint));
            };
            
            if structure.node_parent(point.node_index).is_none() {
                // Transfer to regular AdjustGrab since no parent.
                commands.send_event(ActionEvent::Cancel);
                commands.send_event(ActionEvent::AdjustGrab);
                return;
            }

            let j_transform = transform_query.get(joint).unwrap().translation;

            let p_pos = cam.world_to_viewport(cam_transform, j_transform + axis.to_vec());
            let j_pos = cam.world_to_viewport(cam_transform, j_transform);

            if p_pos.is_err() || j_pos.is_err() {
                return;
            }

            let dir_vec = j_pos.unwrap()-p_pos.unwrap();
            let mouse_move = Vec2::new(-mouse_move.x, -mouse_move.y);
            let mv = mouse_move.dot(dir_vec.normalize());

            let mut transform = transform_query.get_mut(joint).unwrap();

            transform.translation = transform.translation + (mv * 0.02) * axis.to_vec();

            commands.send_event(CursorControlEvent::Position(j_pos.unwrap()));
        },
        EditMode::AdjustRotateAxis(joint, axis) => {
            let Ok(window) = window_q.get_single() else {
                panic!("Window not found");
            };

            let Some(mouse_pos) = window.cursor_position() else {
                return
            };
            
            let ray = cam.viewport_to_world(cam_transform, mouse_pos).unwrap();

            let Ok(point) = joint_query.get(joint) else {
                panic!("{}", Errors::ComponentMissing("Joint", joint));
            };
            
            if structure.node_parent(point.node_index).is_none() {
                // Transfer to regular AdjustGrab since no parent.
                commands.send_event(ActionEvent::Cancel);
                commands.send_event(ActionEvent::AdjustGrab);
                return;
            }
    
            let parent = structure.node_parent_entity(point.node_index).unwrap();
            
            let p = transform_query.get(parent).unwrap().translation;
            
            let mut j_transform = transform_query.get_mut(joint).unwrap();
            let j = j_transform.translation;

            let relative_pos = j-p;
            
            if relative_pos.cross(axis.to_vec()) == Vec3::ZERO { // No rotation possible
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