use bevy::{prelude::*, input::{mouse::MouseMotion}};
use crate::{
    points::*,
    util::*,
};
use super::*;

/// System to handle the movement of the joints when in Grab/Rotate edit modes.
/// 
/// *passive
pub fn grab_control(
    mouse_input: Res<Input<MouseButton>>,
    key_input: Res<Input<KeyCode>>,
    mut motion_evr: EventReader<MouseMotion>,
    images: Res<Assets<Image>>,
    mut windows: ResMut<Windows>,
    is_adjust_mode: Res<IsAdjustMode>,
    joint_selected: Res<JointSelected>,
    pos_cache: Res<PositionCache>,
    mut mv_cache: ResMut<MovementCache>,
    cam_query: Query<(&Camera, &GlobalTransform), With<PerspectiveProjection>>,
    mut editable_query: Query<&mut Editable>,
    mut joint_query: Query<&mut Joint>,
    mut transform_query: Query<&mut Transform>,
    global_query: Query<&mut GlobalTransform, Without<Camera>>,
) {
    if !is_adjust_mode.0 || joint_selected.0.is_none() {
        return;
    } else if mouse_input.just_pressed(MouseButton::Left) || key_input.just_pressed(KeyCode::Escape) {
        let window = windows.get_primary_mut().unwrap();
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
        return;
    }
    let joint = joint_selected.0.unwrap();
    let mut editable = editable_query.get_mut(joint).unwrap();
    let (cam, cam_transform) = cam_query.single();

    let mut mouse_move = Vec2::ZERO;
    
    for ev in motion_evr.iter() {
        mouse_move += ev.delta;
    }
    // println!("processing {:?}", joint_selected.0.unwrap().id());
    let editable_mode = if editable.mode.is_some() {
        editable.mode.as_ref().unwrap()
    } else {
        println!("Editable mode missing for entity {:?}", joint_selected.0.unwrap().id());
        return;
    };
    match editable_mode {
        EditMode::GrabExtend => {
            let mut point = joint_query.get_mut(joint).unwrap();
            
            if point.parent.is_none() {
                editable.mode = Some(EditMode::GrabFull);
                return;
            }

            let parent = point.parent.unwrap();
            
            let p_transform = global_query.get(parent).unwrap();
            let j_transform = global_query.get(joint).unwrap();
            let p_pos = cam.world_to_screen(&windows, &images, cam_transform, p_transform.translation);
            let s_pos = cam.world_to_screen(&windows, &images, cam_transform, j_transform.translation);
            if p_pos.is_none() || s_pos.is_none() {
                return;
            }

            let dir_vec = s_pos.unwrap()-p_pos.unwrap();
            let mouse_move = Vec2::new(mouse_move.x, -mouse_move.y); // mouse_move.y seems to be inverted
            let mv = mouse_move.dot(dir_vec.normalize());

            let mut transform = transform_query.get_mut(joint).unwrap();

            let mut pos = transform.translation + ((mv * 0.01) * transform.translation.normalize());
            mv_cache.0 += mv * 0.01;

            if key_input.pressed(KeyCode::LControl) {
                pos = (mv_cache.0/2.0).round()*2.0 * transform.translation.normalize();
            }

            // prevents extension to negative
            if transform.translation.dot(pos - 2.0 * transform.translation.normalize()) < 0.0  {
                pos = 2.0 * transform.translation.normalize();
                mv_cache.0 = 2.0;
            }
            
            transform.translation = pos;
            let len = transform.translation.length();
            point.dist = len; // update internal length

            let window = windows.get_primary_mut().unwrap();
            window.set_cursor_lock_mode(true);
            window.set_cursor_visibility(false);
            window.set_cursor_position(s_pos.unwrap());
        },
        // both has similar logic
        EditMode::RotateFull | EditMode::GrabFull => {
            let cursor = windows.get_primary().unwrap().cursor_position();
            let mouse_pos = if let Some(cursor) = cursor {
                cursor
            } else {
                return
            };
            
            let ray = ray_from_screenspace(
                mouse_pos, 
                &windows, 
                cam, 
                cam_transform
            ).unwrap();

            let mut point = joint_query.get_mut(joint).unwrap();

            // no parent = root joint.
            let parent = if point.parent.is_some() {
                point.parent.unwrap()
            } else {
                joint
            };

            let parent_global = global_query.get(parent).unwrap().translation;
            let joint_global = global_query.get(joint).unwrap().translation;
            
            // https://antongerdelan.net/opengl/raycasting.html
            let radius = (joint_global-parent_global).length();
            let rot_to_ray = ray.origin() - parent_global;
            let joint_to_ray = ray.origin() - joint_global;
            let b = ray.direction().dot(rot_to_ray);
            let c = rot_to_ray.dot(rot_to_ray) - radius * radius;
            // rotation is spherical when within radius
            let len = if b*b - c >= 0. && editable.mode.as_ref().unwrap() != &EditMode::GrabFull {
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
                let b = joint_global-ray.origin();
                let c = ray.direction();
                // length of c at which b - c is orthogonal to a, just (Cos0 = A / H) in vectors
                (a.dot(b)/a.length()) / (a.dot(c)/(a.length()*c.length()))
            };
            
            let mut joint_local = transform_query.get_mut(joint).unwrap();

            let ray_pos = ray.direction() * len;
            let mouse_pos = ray.origin() + ray_pos;
            if let Some(EditMode::GrabFull) = editable.mode {
                let dir_vec = mouse_pos-parent_global;
                // let pos_c = pos_cache.0;
                // let dir_vec = Vec3::new(dir_vec.x, pos_c.y, pos_c.z);
                joint_local.translation = dir_vec;
                point.dist = dir_vec.length(); 
            } else {
                let dir_vec = (mouse_pos-parent_global).normalize();
                joint_local.translation = point.dist * dir_vec;
            }
        },
        EditMode::GrabAxis(axis) => {
            let mut point = joint_query.get_mut(joint).unwrap();
            
            if point.parent.is_none() {
                editable.mode = Some(EditMode::GrabFull);
                return;
            }

            let j_transform = global_query.get(joint).unwrap();
            let p_pos = cam.world_to_screen(&windows, &images, cam_transform, j_transform.translation + axis.to_vec());
            let s_pos = cam.world_to_screen(&windows, &images, cam_transform, j_transform.translation);
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
            let len = transform.translation.length();
            point.dist = len; // update internal length

            let window = windows.get_primary_mut().unwrap();
            window.set_cursor_lock_mode(true);
            window.set_cursor_visibility(false);
            window.set_cursor_position(s_pos.unwrap());
        },
        // EditMode::RotateAxis(axis) => {
        //     let cursor = windows.get_primary().unwrap().cursor_position();
        //     let mouse_pos = if cursor.is_some() {
        //         cursor.unwrap()
        //     } else {
        //         return
        //     };
            
        //     let ray = ray_from_screenspace(
        //         mouse_pos, 
        //         &windows, 
        //         cam, 
        //         cam_transform
        //     ).unwrap();

        //     let point = joint_query.get_mut(joint).unwrap();
            
        //     if point.parent.is_none() {
        //         editable.mode = Some(EditMode::GrabFull);
        //         return;
        //     }

        //     let parent = point.parent.unwrap();
            
        //     // p + a * (j-p).dot(a)
        //     let p_transform = global_query.get(parent).unwrap();
        //     let j_transform = global_query.get(joint).unwrap();
        //     let mid = p_transform.translation + axis.to_vec() * (pos_cache.0 - p_transform.translation).dot(axis.to_vec());
        //     // let p_pos = cam.world_to_screen(&windows, cam_transform, mid);
        //     // let s_pos = cam.world_to_screen(&windows, cam_transform, j_transform.translation);
        //     // if p_pos.is_none() || s_pos.is_none() {
        //     //     return;
        //     // }
        //     let a = axis.to_vec();
        //     let b = pos_cache.0-ray.origin();
        //     let c = ray.direction();
        //     // length of c at which b - c is orthogonal to a, just (Cos0 = A / H) in vectors
        //     let len = (a.dot(b)/a.length()) / (a.dot(c)/(a.length()*c.length()));
        //     // println!("meet {:?} at {:?}", a, len);

        //     let dir_vec = (ray.origin() + (ray.direction() * len)) - p_transform.translation;
        //     let dir_vec = axis.to_vec().cross(dir_vec.cross(axis.to_vec())) + mid;

        //     // let proj_p = axis.to_vec().cross(pos_cache.0-p_transform.translation.cross(axis.to_vec()));
            
        //     // let angle = if proj_p.cross(dir_vec).normalize() == axis.to_vec() {
        //     //     proj_p.angle_between(dir_vec)
        //     // } else {
        //     //     -proj_p.angle_between(dir_vec)
        //     // };

        //     // let mouse_move = Vec2::new(mouse_move.x, -mouse_move.y); // mouse_move.y seems to be inverted
        //     // let mv = mouse_move.dot(Vec2::new(dir_vec.y, -dir_vec.x)); // ortho dir_vec

        //     let mut transform = transform_query.get_mut(joint).unwrap();

        //     let rotation = Quat::from_rotation_arc(pos_cache.0.normalize(), dir_vec.normalize());
        //     // let rotation = Quat::from_axis_angle(axis.to_vec(), angle);

        //     transform.translation = rotation * pos_cache.0;
        // },
        _ => (),
    }
}

// passive
/// automatically updates the rotation and scaling of the connector when joint location
/// is updated
pub fn update_connector(
    changed_joints: Query<(&Joint, &Transform), Changed<Transform>>,
    mut transform_q: Query<&mut Transform, Without<Joint>>,
) {
    for (joint, transform) in changed_joints.iter() {
        // println!("REEE");
        if joint.rotator.is_none() {
            continue;
        }
        let mut rotator = transform_q.get_mut(joint.rotator.unwrap()).unwrap();
        rotator.rotation = Quat::from_rotation_arc(Vec3::Y, transform.translation.normalize());
        
        let mut connector = transform_q.get_mut(joint.connector.unwrap()).unwrap();

        let scale = Vec3::from([1.0, joint.dist/2.0, 1.0]);
        let rotation = Quat::default();
        let translation = Vec3::new(0.0, joint.dist/2.0, 0.0);
        *connector = Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, translation));
    }
}