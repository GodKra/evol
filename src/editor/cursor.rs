use bevy::prelude::*;
use bevy_mod_picking::prelude::PointerInteraction;
use super::*;
use crate::{util::{JointMaterial, JointMeshes, Errors}, pgraph::*};
/// System to handle the control of the joint addition cursor
/// 
/// *Passive
pub fn cursor_control(
    mut pgraph: ResMut<PGraph>,
    mut commands: Commands,
    mut gizmo: Gizmos,
    joint_materials: Res<JointMaterial>,
    joint_meshes: Res<JointMeshes>,
    mut is_adjust_mode: ResMut<IsAdjustMode>,
    mut selection_updated: ResMut<SelectionUpdated>,
    mouse_input: Res<Input<MouseButton>>,
    mut entity_selected: ResMut<EntitySelected>,
    joint_q: Query<&Joint>,
    mut editable_q: Query<&mut Editable>,
    interaction_q: Query<&PointerInteraction>,
) {
    if !entity_selected.is_joint() {
        return;
    }
    let joint = entity_selected.get().unwrap();
    let Ok(mut editable) = editable_q.get_mut(joint) else {
        panic!("{}", Errors::ComponentMissing("Editable", joint))
    };

    let Some(EditMode::Cursor) = editable.mode else {
        return;
    };

    for interaction in interaction_q.iter() {
        if let Some((target, hit)) = interaction.get_nearest_hit() {
            if *target == joint {
                let hit_pos = hit.position.unwrap();
                let hit_normal = hit.normal.unwrap();
                
                gizmo.ray(hit_pos, hit_normal * hit.depth/4.0, Color::rgb(1.0, 0.0, 0.0));

                // Create new joint & transfer to grab mode
                if mouse_input.just_pressed(MouseButton::Left) {
                    editable.mode = None; // resets editable mode of parent

                    let len = 2.0; // default extension

                    let new_joint = create_joint(
                        &mut commands, 
                        &joint_meshes, 
                        &joint_materials, 
                        hit_pos + hit_normal * len, 
                        None,
                        Editable { mode: Some(EditMode::GrabExtend) },
                        crate::Editor
                    );

                    let parent_data = joint_q.get(joint).unwrap();

                    let node = pgraph.0.add_node(
                        Point { 
                            entityid: Some(new_joint), 
                            parent: Some(parent_data.node_index),
                            pos: hit_pos + hit_normal * len,
                        }
                    );
                    commands.entity(new_joint).insert(Joint { node_index: node });

                    let connector = create_connector(
                        &mut commands, 
                        &joint_meshes, 
                        &joint_materials, 
                        hit_pos + hit_normal * len, 
                        hit_pos + hit_normal * -crate::util::JOINT_RADIUS, 
                        None,
                        (),
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
                    is_adjust_mode.0 = true;
                }
            }
        }
    }
}