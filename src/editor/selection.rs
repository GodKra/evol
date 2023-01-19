use bevy::prelude::*;
use bevy_mod_picking::*;

use crate::util::{JointMaterial};

use super::*;

const MANAGE_SELECT_STG: &str = "manage_selection_stage";

pub const JOINT_SELECT: &str = "joint_select";
pub const S_TYPE_UPDATE: &str = "selection_type_update";
pub const S_HIGHLIGHT: &str = "selection_highlight";

pub struct SelectionPlugin;
impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntitySelected>()
            .init_resource::<SelectionUpdated>()
            .init_resource::<SelectionMaterials>()

            .add_system(
                joint_select
                .run_in_state(crate::GameState::Editor)
                .label(JOINT_SELECT)
                .after(MODE_TOGGLE)
                .run_if(|input: Res<Input<MouseButton>>| {
                    MouseControls::EINTERACT.pressed(input)
                })
            )

            .add_stage_after(CoreStage::Update, MANAGE_SELECT_STG, SystemStage::single_threaded())
            .add_system_to_stage(
                MANAGE_SELECT_STG, 
                update_selection_type
                    .run_in_state(crate::GameState::Editor)
                    .label(S_TYPE_UPDATE))
            .add_system_to_stage(
                MANAGE_SELECT_STG, 
                highlight_selection
                    .run_in_state(crate::GameState::Editor)
                    .label(S_HIGHLIGHT)
                    .after(S_TYPE_UPDATE))
            ;
    }
}

// Colors
#[derive(Resource)]
pub struct SelectionMaterials {
    pub parent_color: Handle<StandardMaterial>,
    pub child_color: Handle<StandardMaterial>,
    pub connector_color: Handle<StandardMaterial>,
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
            connector_color: materials.add(
                StandardMaterial {
                    base_color: Color::rgb(1.0, 0.29, 0.1),
                    emissive: Color::rgba_linear(1.0, 0.29, 0.06, 0.0),
                    ..default()
                }
            ),
        }
    }
}


#[derive(Debug, Clone)]
pub enum SelectableEntity {
    Joint(Entity),
    Connector(Entity)
    // Muscle,
}

#[derive(Debug)]
pub enum SelectionMode {
    JointParent,
    JointChild,
    Connector,
}

/// Describes selection type. None = not selected.
#[derive(Component)]
pub struct Selectable {
    pub entity_type: SelectableEntity,
    pub select_mode: Option<SelectionMode>,
}

impl Selectable {
    pub fn with_type(entity_type: SelectableEntity) -> Self {
        Selectable {
            entity_type,
            select_mode: None,
        }
    }
}

/// Currently selected Entity. If SelectionUpdated is true then the entity set will be highlighted.
#[derive(Default, Resource)]
pub struct EntitySelected(pub Option<SelectableEntity>);

impl EntitySelected {
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
    pub fn set(&mut self, entity: Option<SelectableEntity>) {
        self.0 = entity
    }
    pub fn contains(&self, entity: Entity) -> bool {
        if let Some(selected) = &self.0 {
            return match selected {
                SelectableEntity::Joint(val) => entity == *val,
                SelectableEntity::Connector(val) => entity == *val,
            }
        }
        false
    }
    /// Returns the entity selected if any.
    pub fn get(&self) -> Option<Entity> {
        if let Some(selected) = &self.0 {
            return match selected {
                SelectableEntity::Joint(val) => Some(*val),
                SelectableEntity::Connector(val) => Some(*val),
            }
        }
        None
    }
    pub fn is_joint(&self) -> bool {
        if let Some(SelectableEntity::Joint(_)) = &self.0 {
            return true
        }
        false
    }
    pub fn is_connector(&self) -> bool {
        if let Some(SelectableEntity::Connector(_)) = &self.0 {
            return true
        }
        false
    }
}

/// Set to true if any changes are to be made to the selection
#[derive(Default, Resource)]
pub struct SelectionUpdated(pub bool);

/// System to set the joint_selected resource when mouse clicked
/// 
/// *active
fn joint_select(
    mut entity_selected: ResMut<EntitySelected>,
    mut selection_updated: ResMut<SelectionUpdated>,
    selectable_q: Query<&Selectable>,
    pick_cam: Query<&PickingCamera>,
) {
    // this should always work
    let cam = pick_cam.single();
    
    if let Some((selected, _)) = cam.get_nearest_intersection() {
        // does not run if selection has just been updated (for joint creation through cursor)
        if entity_selected.contains(selected) || selection_updated.0 {
            return;
        }

        let selectable = selectable_q.get(selected).unwrap();
        entity_selected.set(Some(selectable.entity_type.clone()));
        selection_updated.0 = true;
        
        println!(":: Selected: {:?}", entity_selected.0);
    } else if entity_selected.is_some() {
        selection_updated.0 = true;
        entity_selected.set(None);
        
        println!(":: Selected: {:?}", entity_selected.0);
    }
}

/// System to update the current joint_selected and its children's Selectable component.
/// 
/// *passive
fn update_selection_type(
    // joint_selected: Res<JointSelected>,
    // conn_selected: Res<ConnectorSelected>,
    entity_selected: Res<EntitySelected>,
    selection_updated: Res<SelectionUpdated>,
    mut selectable_query: Query<&mut Selectable>,
    // mut changed_selectable_query: Query<&mut Selectable, Changed<Selectable>>,
    child_query: Query<&Children>,
) {
    if !selection_updated.0 {
        return;
    }
    
    for mut selectable in selectable_query.iter_mut() {
        selectable.select_mode = None;
    }
    if !entity_selected.is_some() {
        return;
    }

    match entity_selected.0.as_ref().unwrap() { // determines the behaviour of the highlight system.
        SelectableEntity::Joint(joint) => {
            select_joints_recursive(&joint, true, &mut selectable_query, &child_query)
        },
        SelectableEntity::Connector(conn) => {
            let mut selectable = selectable_query.get_mut(*conn).unwrap();
            selectable.select_mode = Some(SelectionMode::Connector);
        },
    }
}

/// System to update entity highlight based on its select_mode
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
            if let Some(select_type) = &selectable.select_mode {
                match select_type { // Highlight selected
                    SelectionMode::JointParent => *material_handle = select_materials.parent_color.clone(),
                    SelectionMode::JointChild => *material_handle = select_materials.child_color.clone(),
                    SelectionMode::Connector => *material_handle = select_materials.connector_color.clone(),
                }
            } else {
                match &selectable.entity_type { // Reset highlight for those not selected
                    &SelectableEntity::Joint(_) => *material_handle = joint_materials.joint_color.clone(),
                    &SelectableEntity::Connector(_) => *material_handle = joint_materials.connector_color.clone(),
                }
            }
        }
        selection_updated.0 = false;
    }
}

fn select_joints_recursive(
    joint: &Entity,
    is_parent: bool,
    selectable_query: &mut Query<&mut Selectable>,
    child_query: &Query<&Children>,
) {
    // let mut selectable = if let Ok(selectable) = selectable_query.get_mut(*joint) {
    //     selectable
    // } else {
    //     return;
    // };
    let mut selectable = selectable_query.get_mut(*joint).unwrap();
    selectable.select_mode = if is_parent {
        Some(SelectionMode::JointParent)
    } else {
        Some(SelectionMode::JointChild)
    };
 
    let children = get_selectable_children(joint, selectable_query, child_query);
    for child in children {
        select_joints_recursive(&child, false, selectable_query, child_query);
    }
}

/// Returns a list of immediate children with the Selectable component.
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
            }
        }
    }
    selectable
}