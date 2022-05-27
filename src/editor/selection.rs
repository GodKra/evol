use bevy::prelude::*;
use bevy_mod_picking::*;

use crate::points::JointMaterial;

use super::*;

const MANAGE_SELECT_STG: &str = "manage_selection_stage";

pub const JOINT_SELECT: &str = "joint_select";
pub const S_TYPE_UPDATE: &str = "selection_type_update";
pub const S_HIGHLIGHT: &str = "selection_highlight";

pub struct SelectionPlugin;
impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JointSelected>()
            .init_resource::<SelectionUpdated>()
            .init_resource::<SelectionMaterials>()

            .add_stage_after(CoreStage::Update, MANAGE_SELECT_STG, SystemStage::single_threaded())
            
            .add_system(
                joint_select
                    .label(JOINT_SELECT)
                    .after(MODE_TOGGLE))
            .add_system_to_stage(
                MANAGE_SELECT_STG, 
                update_selection_type
                    .label(S_TYPE_UPDATE))
            .add_system_to_stage(
                MANAGE_SELECT_STG, 
                highlight_selection
                    .label(S_HIGHLIGHT)
                    .after(S_TYPE_UPDATE))
            ;
    }
}

// Colors
pub struct SelectionMaterials {
    pub parent_color: Handle<StandardMaterial>,
    pub child_color: Handle<StandardMaterial>,
}

impl FromWorld for SelectionMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        SelectionMaterials {
            parent_color: materials.add(
                StandardMaterial {
                    base_color: Color::rgb(1., 0.858, 0.301),
                    emissive: Color::rgba_linear(0.521, 0.415, 0.0, 0.0),
                    ..default()
                }
            ),
            child_color: materials.add(
                StandardMaterial {
                    base_color: Color::rgb(0.874, 0.262, 0.003),
                    emissive: Color::rgba_linear(0.180, 0.054, 0.0, 0.0),
                    ..default()
                }
            ),
        }
    }
}


#[derive(Debug)]
pub enum SelectedType {
    Parent,
    Child,
}

/// Describes selection type. None = not selected.
#[derive(Default, Component)]
pub struct Selectable {
    pub selected: Option<SelectedType>,
}

/// Currently selected joint. 
/// if SelectionUpdated is set to true, changes to this will result in its children being selected and highlighted.
#[derive(Default, Debug)]
pub struct JointSelected(pub Option<Entity>);

/// Set to true if any changes need to be made to the selection
#[derive(Default)]
pub struct SelectionUpdated(pub bool);

/// System to set the joint_selected resource when mouse clicked
/// 
/// *active
fn joint_select(
    mut joint_selected: ResMut<JointSelected>,
    mut selection_updated: ResMut<SelectionUpdated>,
    mouse_input: Res<Input<MouseButton>>,
    pick_cam: Query<&PickingCamera>,
) {
    // click only once
    if !mouse_input.just_pressed(MouseButton::Left) {
        return
    }

    // this should always work
    let cam = pick_cam.single();
    
    if let Some((joint, _)) = cam.intersect_top() {
        if joint_selected.0.is_some() && (joint_selected.0.unwrap() == joint || selection_updated.0) {
            return;
        }
        joint_selected.0 = Some(joint);
        selection_updated.0 = true;
        
        println!("DEBUG: Joint: {:?}", joint_selected.0);
    } else if joint_selected.0.is_some() {
        selection_updated.0 = true;
        joint_selected.0 = None;
        
        println!("DEBUG: Joint: {:?}", joint_selected.0);
    }
}

/// System to update the current joint_selected and its children's Selectable component.
/// 
/// *passive
fn update_selection_type(
    joint_selected: Res<JointSelected>,
    selection_updated: Res<SelectionUpdated>,
    mut selectable_query: Query<&mut Selectable>,
    // mut changed_selectable_query: Query<&mut Selectable, Changed<Selectable>>,
    child_query: Query<&Children>,
) {
    if !selection_updated.0 {
        return;
    }
    for mut selectable in selectable_query.iter_mut() {
        selectable.selected = None;
    }
    
    if joint_selected.0.is_some() {
        select_joints_recursive(&joint_selected.0.unwrap(), true, &mut selectable_query, &child_query);
    }
}

fn select_joints_recursive(
    joint: &Entity,
    is_parent: bool,
    selectable_query: &mut Query<&mut Selectable>,
    child_query: &Query<&Children>,
) {
    let mut selectable = selectable_query.get_mut(*joint).unwrap();
    selectable.selected = if is_parent {
        Some(SelectedType::Parent)
    } else {
        Some(SelectedType::Child)
    };
 
    let children = get_selectable_children(joint, selectable_query, child_query);
    for child in children {
        select_joints_recursive(&child, false, selectable_query, child_query);
    }
}

fn get_selectable_children(
    joint: &Entity,
    selectable_query: &mut Query<&mut Selectable>,
    child_query: &Query<&Children>
) -> Vec<Entity> {
    let mut selectable = Vec::new();
    let c_children = child_query.get(*joint);
    if let Ok(children) = c_children {
        for child in children.iter() {
            if selectable_query.get(*child).is_ok() {
                selectable.push(*child);
            }// else { // used when joints were parented to rotator and rotator to parent
            //     let mut c = get_selectable_children(child, selectable_query, child_query);
            //     selectable.append(&mut c);
            // }
        }
    }
    selectable
}

/// System to update joint highlight based on its SelectedType
/// 
/// passive
fn highlight_selection(
    mut selection_updated: ResMut<SelectionUpdated>,
    select_materials: Res<SelectionMaterials>,
    joint_materials: Res<JointMaterial>,
    mut s_query: Query<(&mut Handle<StandardMaterial>, &Selectable)>,
) {
    if selection_updated.0 {
        for (mut material_handle, selectable) in s_query.iter_mut() {
            if let Some(select_type) = &selectable.selected {
                match select_type {
                    SelectedType::Parent => *material_handle = select_materials.parent_color.clone(),
                    SelectedType::Child => *material_handle = select_materials.child_color.clone(),
                }
            } else {
                *material_handle = joint_materials.joint_color.clone();
            }
        }
        selection_updated.0 = false;
    }
}