use bevy::prelude::*;
use bevy_mod_raycast::Ray3d;

use super::{*, selection::JointSelected, joint::Joint};

pub const DOF_DISTANCE: f32 = 1.2;
pub const DOF_SCALE: f32 = 0.1;

#[derive(Component)]
pub struct DOFPointer;


/// System to change/set the degrees of freedom for the selected joint.
/// to create muscles.
/// 
/// * Passive
pub fn set_dof(
    windows: Res<Windows>,
    meshes: Res<JointMeshes>,
    mouse_input: Res<Input<MouseButton>>,
    key_input: Res<Input<KeyCode>>,
    joint_selected: ResMut<JointSelected>,
    mut mesh_q: Query<&mut Handle<Mesh>, With<DOFPointer>>,
    cam_query: Query<(&Camera, &GlobalTransform)>,
    mut editable_q: Query<&mut Editable>,
    gtransform_q: Query<&GlobalTransform>,
    mut pointer_q: Query<&mut Transform, With<DOFPointer>>,
    mut joint_q: Query<&mut Joint>,
) {
    if joint_selected.0.is_none() {
        return;
    }
    let joint = joint_selected.0.unwrap();
    let mut editable = if let Ok(editable) = editable_q.get_mut(joint) {
        editable
    } else {
        return
    };

    let (cam, cam_transform) = cam_query.single();

    if let Some(EditMode::AOF) = editable.mode {
        let mut jointdata = joint_q.get_mut(joint).unwrap();
        if jointdata.dof_pointer.is_none() {
            return;
        }

        if key_input.just_pressed(KeyCode::F) {
            let mut mesh_handle = mesh_q.get_mut(jointdata.dof_pointer.unwrap()).unwrap();
            jointdata.locked = !jointdata.locked;
            if jointdata.locked {
                *mesh_handle = meshes.dof_locked.clone();
            } else {
                *mesh_handle = meshes.dof_free.clone();
            }
        }

        let mut pointer_t = pointer_q.get_mut(jointdata.dof_pointer.unwrap()).unwrap();

        let cursor = windows.get_primary().unwrap().cursor_position();

        let mouse_pos = if let Some(cursor) = cursor {
            cursor
        } else {
            return
        };
        
        let ray = Ray3d::from_screenspace(
            mouse_pos,
            cam, 
            cam_transform
        ).unwrap();

        let normal = (pointer_t.rotation * Vec3::Y).normalize();
        let joint_t = gtransform_q.get(joint).unwrap().translation();

        let local_ray = Ray3d::new(ray.origin()-joint_t, ray.direction()); // ray in joint's local space
        
        let intersection = get_intersect_plane_ray(
            pointer_t.translation, 
            normal, 
            local_ray
        );

        let dir_vec = intersection - pointer_t.translation;
        let pointer_dir = pointer_t.rotation * Vec3::Z;

        let local_y = if normal.y != 0.0 {
            let len = normal.length_squared()/normal.y;
            (Vec3::Y*len - normal).normalize()
        } else {
            Vec3::Y
        };

        let rot = get_axis_rotation(pointer_dir, dir_vec, normal);
        pointer_t.rotation = rot * pointer_t.rotation;

        if mouse_input.just_pressed(MouseButton::Left) {
            editable.mode = None;
            jointdata.dof = get_rotation_angle(local_y, pointer_t.rotation * Vec3::Z, normal);
            println!("dof: {:?}", jointdata.dof);
        }           
    }
}

pub fn position_pointer(
    mut pointer_q: Query<&mut Transform, With<DOFPointer>>,
    joint_selected: Res<JointSelected>,
    joint_q: Query<&Joint>,
    editable_q: Query<&Editable>,
    transform_q: Query<&GlobalTransform>,
) {
    if let Some(joint) = joint_selected.0 {
        let editable = editable_q.get(joint);
        if let Ok(editable) = editable {
            if let Some(EditMode::AOF) = editable.mode {
                return;
            }
        } else {
            return;
        }
        let jointdata = joint_q.get(joint).unwrap();
        if jointdata.dof_pointer.is_none() {
            return;
        }

        // let mut pointer_t = pointer_q.single_mut();
        let mut pointer_t = pointer_q.get_mut(jointdata.dof_pointer.unwrap()).unwrap();
        let r_rot = transform_q.get(jointdata.rotator.unwrap()).unwrap().to_scale_rotation_translation().1;
        // let gtransform = transform_q.get(jointdata.dof_pointer.unwrap()).unwrap();
        
        // let normal = r_rot * Vec3::Y;

        // old
        // let y_side = normal.project_onto(Vec3::Y);
        // let side = normal - y_side;
        // let angle = (y_side.length()/normal.length()).acos();
        // let len = angle.tan() * side.length();

        // let pointer_dir = if normal.y > 0.0 {
        //     (normal - (y_side + Vec3::Y * len)).normalize()
        // } else {
        //     (normal - (y_side - Vec3::Y * len)).normalize()
        // };

        let normal = r_rot * Vec3::Y;
        let local_y = if normal.y != 0.0 {
            let len = normal.length_squared()/normal.y;
            (Vec3::Y*len - normal).normalize()
        } else {
            Vec3::Y
        };

        let pointer_dir = Quat::from_axis_angle(normal, jointdata.dof) * local_y; 

        let forward = pointer_dir.normalize();
        let right = normal.cross(forward).normalize();
        pointer_t.rotation = Quat::from_mat3(&Mat3::from_cols(right, normal, forward));
        pointer_t.translation = r_rot * Vec3::new(0., DOF_DISTANCE, 0.);
    }
}

pub fn pointer_visibility(
    mut pointer_q: Query<&mut Visibility, With<DOFPointer>>,
    joint_q: Query<&Joint>,
    joint_selected: Res<JointSelected>,
    selection_updated: Res<SelectionUpdated>,
) {
    if !selection_updated.0 {
        return
    }
    for mut pointer in pointer_q.iter_mut() {
        pointer.is_visible = false;
    }
    if let Some(joint) = joint_selected.0 {
        let joint = joint_q.get(joint);
        if let Ok(joint) = joint {
            if joint.dof_pointer.is_some() {
                let mut selected_pointer = pointer_q.get_mut(joint.dof_pointer.unwrap()).unwrap();
                selected_pointer.is_visible = true;
            }
        }
    }
}

