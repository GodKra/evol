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
    images: Res<Assets<Image>>,
    windows: Res<Windows>,
    meshes: Res<JointMeshes>,
    mouse_input: Res<Input<MouseButton>>,
    key_input: Res<Input<KeyCode>>,
    joint_selected: ResMut<JointSelected>,
    mut mesh_q: Query<&mut Handle<Mesh>, With<DOFPointer>>,
    cam_query: Query<(&Camera, &GlobalTransform), With<PerspectiveProjection>>,
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

        if key_input.just_pressed(KeyCode::F) {
            let mut mesh_handle = mesh_q.get_mut(jointdata.dof_pointer.unwrap()).unwrap();
            if jointdata.locked {
                *mesh_handle = meshes.dof_free.clone();
            } else {
                *mesh_handle = meshes.dof_locked.clone();
            }
            jointdata.locked = !jointdata.locked;
        }

        let mut pointer_t = pointer_q.get_mut(jointdata.dof_pointer.unwrap()).unwrap();

        let cursor = windows.get_primary().unwrap().cursor_position();

        let mouse_pos = if let Some(cursor) = cursor {
            cursor
        } else {
            return
        };
        
        let ray = ray_from_screenspace(
            mouse_pos, 
            &windows, 
            &images,
            cam, 
            cam_transform
        ).unwrap();

        let normal = (pointer_t.rotation * Vec3::Y).normalize();
        let joint_t = gtransform_q.get(joint).unwrap();

        let local_ray = Ray3d::new(ray.origin()-joint_t.translation, ray.direction()); // ray in joint's local space
        
        let intersection = get_intersect_plane_ray(
            pointer_t.translation, 
            normal, 
            local_ray
        );

        let dir_vec = intersection - pointer_t.translation;
        let pointer_dir = pointer_t.rotation * Vec3::Z;

        let rot = get_axis_rotation(pointer_dir, dir_vec, normal);
        pointer_t.rotation = rot * pointer_t.rotation;

        if mouse_input.just_pressed(MouseButton::Left) {
            println!("here");
            editable.mode = None;
            jointdata.dof = dir_vec;
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

        // let mut pointer_t = pointer_q.single_mut();
        let jointdata = joint_q.get(joint).unwrap();
        let mut pointer_t = pointer_q.get_mut(jointdata.dof_pointer.unwrap()).unwrap();
        let rotator_t = transform_q.get(jointdata.rotator.unwrap()).unwrap();
        pointer_t.translation = rotator_t.rotation * Vec3::new(0., DOF_DISTANCE, 0.);
        
        let normal = rotator_t.rotation * Vec3::Y;
        // theoretical twist nullfication -- doesnt work
        // let normal_ortho = normal.any_orthonormal_vector();
        // let normal_rot = rotator_t.rotation * normal_ortho;
        // let normal_proj = normal.cross(normal_rot.cross(normal));
        // let anti_rot = get_axis_rotation(normal_proj, normal_ortho, normal);
        // let anti_rot = Quat::from_axis_angle(normal, 2.14);
        
        let pointer_dir = rotator_t.rotation * Vec3::Z;
        let rot = get_axis_rotation(pointer_dir, jointdata.dof, normal);
        // pointer_t.rotation = rot * anti_rot * rotator_t.rotation;
        pointer_t.rotation = rot *  rotator_t.rotation;
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
            let mut selected_pointer = pointer_q.get_mut(joint.dof_pointer.unwrap()).unwrap();
            selected_pointer.is_visible = true;
        }
    }
}

