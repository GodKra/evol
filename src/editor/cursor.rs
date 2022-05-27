use bevy::{prelude::*};
use bevy_mod_picking::PickingCamera;

use crate::{
    points::*,
};
use super::*;

/// System to handle the control of the joint addition cursor
/// 
/// *Passive
pub fn cursor_control(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    joint_materials: Res<JointMaterial>,
    joint_meshes: Res<JointMeshes>,
    mut is_adjust_mode: ResMut<IsAdjustMode>,
    mut selection_updated: ResMut<SelectionUpdated>,
    mouse_input: Res<Input<MouseButton>>,
    mut joint_selected: ResMut<JointSelected>,
    added_pick_cam: Query<&PickingCamera, Added<PickingCamera>>,
    pick_cam: Query<&PickingCamera>,
    mut editable_query: Query<&mut Editable>,
    mut cursor_query: Query<&mut GlobalTransform, With<EditCursor>>,
    mut visibility_query: Query<&mut Visibility, With<EditCursor>>,
) {
    
    let cube_size = 0.02;
    let cube_tail_scale = 20.0;
    
    // spawn the cursor (should run just once)
    for _ in added_pick_cam.iter() {
        let mut transform = Transform::from_translation(Vec3::new(
            0.0,
            (cube_size * cube_tail_scale) / 2.0,
            0.0,
        ));
        transform.apply_non_uniform_scale(Vec3::from([1.0, cube_tail_scale, 1.0]));

        let cursor_material = &materials.add(StandardMaterial {
            base_color: Color::rgb(1.0, 0.0, 0.0),
            unlit: true,
            ..Default::default()
        });
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: cube_size })),
            material: cursor_material.clone(),
                transform,
                ..Default::default()
            })
            .insert(EditCursor::default());
        println!("cursor added");
    }

    if joint_selected.0.is_none() {
        return;
    }
    let joint = joint_selected.0.unwrap();
    let mut editable = editable_query.get_mut(joint).unwrap();

    if let Some(EditMode::Cursor) = editable.mode {
        let cam = pick_cam.single();
        if let Some((target, intersection)) = cam.intersect_top() {
            if target == joint {
                // -- cursor code borrowed from bevy-mod-picking -- 
                // println!("ignored target: {} \nselected: {}", target.id(), joint.id() );
                let transform_new = intersection.normal_ray().to_transform();

                for mut transform in cursor_query.iter_mut() {
                    let scale = Vec3::from([
                        (intersection.distance()/2.0)*0.5, 
                        cube_tail_scale * (intersection.distance()/2.0), 
                        (intersection.distance()/2.0)*0.5
                    ]);
                    let rotation = Quat::default();
                    let translation = Vec3::new(0.0, (cube_size * cube_tail_scale * (intersection.distance()/2.0)) / 2.0, 0.0);
                    let transform_move =
                        Mat4::from_scale_rotation_translation(scale, rotation, translation);
                    *transform = GlobalTransform::from_matrix(transform_new * transform_move)
                }
                for mut visible in &mut visibility_query.iter_mut() {
                    visible.is_visible = true;
                }

                // Create new joint & transfer to adjust mode
                if mouse_input.just_pressed(MouseButton::Left) {
                    editable.mode = None; // resets editable mode of parent

                    let len = 2.0; // default extension

                    let joint = create_joint(
                        Some(target), 
                        intersection.normal() * len, 
                        Some(EditMode::GrabExtend),
                        &mut commands, 
                        &joint_meshes, 
                        &joint_materials
                    );
                    
                    joint_selected.0 = Some(joint);
                    selection_updated.0 = true;
                    is_adjust_mode.0 = true;
                } else {
                    return;
                }
            }
        }
    }

    // reached if nothing is intersected. hides the cursor.
    for mut visible in &mut visibility_query.iter_mut() {
        visible.is_visible = false;
    }
}