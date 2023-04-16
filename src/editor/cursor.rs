use bevy::{prelude::*, math::Affine3A};
use bevy_mod_picking::{PickingCamera};
use bevy_mod_raycast::Ray3d;

use super::*;
use crate::{util::{JointMaterial, JointMeshes, Errors}, pgraph::*};
/// System to handle the control of the joint addition cursor
/// 
/// *Passive
pub fn cursor_control(
    mut pgraph: ResMut<PGraph>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    joint_materials: Res<JointMaterial>,
    joint_meshes: Res<JointMeshes>,
    mut is_grab_mode: ResMut<IsAdjustMode>,
    mut selection_updated: ResMut<SelectionUpdated>,
    mouse_input: Res<Input<MouseButton>>,
    mut entity_selected: ResMut<EntitySelected>,
    added_pick_cam: Query<&PickingCamera, Added<PickingCamera>>,
    pick_cam: Query<&PickingCamera>,
    joint_q: Query<&Joint>,
    mut editable_q: Query<&mut Editable>,
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
        transform.scale *= Vec3::from([1.0, cube_tail_scale, 1.0]);

        let cursor_material = &materials.add(StandardMaterial {
            base_color: Color::rgb(1.0, 0.0, 0.0),
            unlit: true,
            ..Default::default()
        });
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: cube_size })),
                material: cursor_material.clone(),
                transform,
                visibility: Visibility::Hidden,
                ..Default::default()
                
            },
            EditCursor::default(),
            crate::Editor
        ));
        println!("** Cursor Added");
    }

    if !entity_selected.is_joint() {
        return;
    }
    let joint = entity_selected.get().unwrap();
    let Ok(mut editable) = editable_q.get_mut(joint) else {
        panic!("{}", Errors::ComponentMissing("Editable", joint))
    };

    if let Some(EditMode::Cursor) = editable.mode {
        let cam = pick_cam.single();
        if let Some((target, intersection)) = cam.get_nearest_intersection() {
            if target == joint {
                let transform_new = Ray3d::new(intersection.position(), intersection.normal()).to_transform();
                

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
                    *transform = Affine3A::from_mat4(transform_new * transform_move).into()
                }
                for mut visible in &mut visibility_query.iter_mut() {
                    *visible = Visibility::Visible;
                }

                // Create new joint & transfer to grab mode
                if mouse_input.just_pressed(MouseButton::Left) {
                    editable.mode = None; // resets editable mode of parent

                    let len = 2.0; // default extension

                    let new_joint = create_joint(
                        &mut commands, 
                        &joint_meshes, 
                        &joint_materials, 
                        intersection.position() + intersection.normal() * len, 
                        None,
                        Editable { mode: Some(EditMode::GrabExtend) },
                        crate::Editor
                    );

                    let parent_data = joint_q.get(joint).unwrap();

                    let node = pgraph.0.add_node(
                        Point { 
                            entityid: Some(new_joint), 
                            parent: Some(parent_data.node_index),
                            pos: intersection.position() + intersection.normal() * len,
                            ..default()
                        }
                    );
                    commands.entity(new_joint).insert(Joint { node_index: node });

                    let connector = create_connector(
                        &mut commands, 
                        &joint_meshes, 
                        &joint_materials, 
                        intersection.position() + intersection.normal() * len, 
                        intersection.position() + intersection.normal() * -crate::util::JOINT_RADIUS, 
                        None,
                        crate::Editor
                    );

                    let edge = pgraph.0.add_edge(node, parent_data.node_index, Connection {
                        entityid: Some(connector),
                        ..default()
                    });
                    commands.entity(connector).insert(Connector{edge_index: edge},);
                    
                    entity_selected.set(Some(SelectableEntity::Joint(new_joint)));
                    println!(":: Created Joint: {:?}", new_joint);
                    selection_updated.0 = true;
                    is_grab_mode.0 = true;
                } else {
                    return;
                }
            }
        }
    }

    // reached if nothing is intersected. hides the cursor.
    for mut visible in &mut visibility_query.iter_mut() {
        *visible = Visibility::Hidden;
    }
}