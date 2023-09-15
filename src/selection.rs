use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
// use iyes_loopless::prelude::*;

use crate::util::JointMaterial;

use crate::pgraph::*;

pub struct SelectionPlugin;
impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntitySelected>()
            .init_resource::<SelectionUpdated>()
            .init_resource::<HighlightMaterials>()

            .add_systems(
                PostUpdate,
                (
                    joint_select,
                    (
                        update_selection_type,
                        highlight_selection
                    ).chain(),
                )
            );
    }
}

// Colors
#[derive(Resource)]
pub struct HighlightMaterials {
    pub joint_color: Handle<StandardMaterial>,
    pub connector_color: Handle<StandardMaterial>,
    pub muscle_color: Handle<StandardMaterial>,
}

impl FromWorld for HighlightMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        HighlightMaterials {
            joint_color: materials.add(
                StandardMaterial {
                    base_color: Color::rgb(1., 0.858, 0.301),
                    emissive: Color::rgba_linear(0.521, 0.415, 0.0, 0.0),
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
            muscle_color: materials.add(
                StandardMaterial {
                    base_color: Color::rgb(1.0, 0.29, 0.1),
                    emissive: Color::rgba_linear(0.0, 1.0, 0.0, 1.0),
                    ..default()
                }
            ),
        }
    }
}


#[derive(Debug, Clone)]
pub enum SelectableEntity {
    Joint(Entity),
    Connector(Entity),
    Muscle(Entity),
}

#[derive(Debug)]
pub enum SelectionMode {
    Joint,
    Connector,
    Muscle,
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
                SelectableEntity::Muscle(val) => entity == *val,
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
                SelectableEntity::Muscle(val) => Some(*val),
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
    pub fn is_muscle(&self) -> bool {
        if let Some(SelectableEntity::Muscle(_)) = &self.0 {
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
pub fn joint_select(
    mut entity_selected: ResMut<EntitySelected>,
    mut selection_updated: ResMut<SelectionUpdated>,
    selectable_q: Query<&Selectable>,
    select_q: Query<(Entity, &PickSelection), Changed<PickSelection>>,
    pointer_q: Query<&PointerInteraction>,
) {
    for (target, select_state) in select_q.iter() {
        let pointer = pointer_q.single();
        if select_state.is_selected {
            if entity_selected.contains(target) || selection_updated.0 {
                return;
            }

            let selectable = selectable_q.get(target).unwrap();
            entity_selected.set(Some(selectable.entity_type.clone()));
            selection_updated.0 = true;
            
            println!(":: Selected: {:?}", entity_selected.0);
        } else if pointer.get_nearest_hit().is_none() {
            selection_updated.0 = true;
            entity_selected.set(None);
            println!(":: Selected: {:?}", entity_selected.0);
        }
    }
}

/// System to update the current joint_selected and its children's Selectable component.
/// 
/// *passive
fn update_selection_type(
    pgraph: Res<PGraph>,
    entity_selected: Res<EntitySelected>,
    selection_updated: Res<SelectionUpdated>,
    mut selectable_query: Query<&mut Selectable>,
    joint_q: Query<&Joint>,
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
            let mut selectable = selectable_query.get_mut(*joint).unwrap();
            selectable.select_mode = Some(SelectionMode::Joint);
            
            select_parent_edge(*joint, &pgraph, &mut selectable_query, &joint_q);
        },
        SelectableEntity::Connector(conn) => {
            let mut selectable = selectable_query.get_mut(*conn).unwrap();
            selectable.select_mode = Some(SelectionMode::Connector);
        },
        SelectableEntity::Muscle(muscle) => {
            let mut selectable = selectable_query.get_mut(*muscle).unwrap();
            selectable.select_mode = Some(SelectionMode::Muscle);
        },
    }
}

/// System to update entity highlight based on its select_mode. Runs over every selectable entity everytime
/// selection is updated. (maybe improvable? TODO)
/// 
/// passive
fn highlight_selection(
    mut selection_updated: ResMut<SelectionUpdated>,
    select_materials: Res<HighlightMaterials>,
    joint_materials: Res<JointMaterial>,
    mut s_query: Query<(&mut Handle<StandardMaterial>, &Selectable)>,
) {
    if selection_updated.0 {
        for (mut material_handle, selectable) in s_query.iter_mut() {
            if let Some(select_type) = &selectable.select_mode {
                match select_type { // Highlight selected
                    SelectionMode::Joint => *material_handle = select_materials.joint_color.clone(),
                    // SelectionMode::JointChild => *material_handle = select_materials.child_color.clone(),
                    SelectionMode::Connector => *material_handle = select_materials.connector_color.clone(),
                    SelectionMode::Muscle => *material_handle = select_materials.muscle_color.clone(),
                }
            } else {
                match selectable.entity_type { // Reset highlight for those not selected
                    SelectableEntity::Joint(_) => *material_handle = joint_materials.joint_color.clone(),
                    SelectableEntity::Connector(_) => *material_handle = joint_materials.connector_color.clone(),
                    SelectableEntity::Muscle(_) => *material_handle = joint_materials.muscle_color.clone(),
                }
            }
        }
        selection_updated.0 = false;
    }
}

fn select_parent_edge(
    joint: Entity,
    pgraph: &Res<PGraph>,
    selectable_query: &mut Query<&mut Selectable>,
    joint_q: &Query<&Joint>,
) {
    let joint_data = joint_q.get(joint).unwrap();
            
    let pdata = pgraph.0.node_weight(joint_data.node_index).unwrap();
    if let Some(parent) = pdata.parent {
        let Some(edge) = pgraph.0.find_edge(joint_data.node_index, parent) else {
            println!("edge missing between parent and joint");
            return;
        };
        let conn = pgraph.edge_to_entity(edge).unwrap();
        let mut selectable = selectable_query.get_mut(conn).unwrap();
        selectable.select_mode = Some(SelectionMode::Connector);
    }
}