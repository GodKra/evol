use bevy::picking::prelude::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::util::JointMaterial;

use crate::structure::*;

pub struct SelectionPlugin;
impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntitySelected>()
            .init_resource::<HighlightMaterials>()
            .add_event::<SelectionUpdateEvent>()
            .add_event::<SelectionBlockEvent>()

            .add_systems(Update, select_on_click)

            .add_systems(PostUpdate, highlight_selection)
            
            .add_observer(update_selectables);
    }
}

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
                    base_color: Color::srgb(1., 0.858, 0.301),
                    emissive: Color::linear_rgba(0.521, 0.415, 0.0, 0.0).into(),
                    unlit: true,
                    ..default()
                }
            ),
            connector_color: materials.add(
                StandardMaterial {
                    base_color: Color::srgb(1.0, 0.29, 0.1),
                    emissive: Color::linear_rgba(1.0, 0.29, 0.06, 0.0).into(),
                    unlit: true,
                    ..default()
                }
            ),
            muscle_color: materials.add(
                StandardMaterial {
                    base_color: Color::srgb(1.0, 0.29, 0.1),
                    emissive: Color::linear_rgba(0.0, 1.0, 0.0, 1.0).into(),
                    unlit: true,
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

#[derive(Component)]
pub struct Selectable {
    pub entity_type: SelectableEntity,
    pub selected: bool,
}

impl Selectable {
    pub fn with_type(entity_type: SelectableEntity) -> Self {
        Selectable {
            entity_type,
            selected: false,
        }
    }
}

/// Currently selected Entity. Will be highlighted when SelectionUpdateEvent is triggered.
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
        matches!(self.0, Some(SelectableEntity::Joint(_)))
        // if let Some(SelectableEntity::Joint(_)) = &self.0 {
        //     return true
        // }
        // false
    }
    pub fn is_connector(&self) -> bool {
        matches!(self.0, Some(SelectableEntity::Connector(_)))
        // if let Some(SelectableEntity::Connector(_)) = &self.0 {
        //     return true
        // }
        // false
    }
    pub fn is_muscle(&self) -> bool {
        matches!(self.0, Some(SelectableEntity::Muscle(_)))
        // if let Some(SelectableEntity::Muscle(_)) = &self.0 {
        //     return true
        // }
        // false
    }
}

#[derive(Event)]
pub struct SelectionUpdateEvent;

#[derive(Event)]
pub struct SelectionBlockEvent;

fn select_on_click(
    mut commands: Commands,
    mut ray_cast: MeshRayCast,
    mouse: Res<ButtonInput<MouseButton>>,
    cam_q: Query<(&Camera, &GlobalTransform)>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut entity_selected: ResMut<EntitySelected>,
    selectable_q: Query<&Selectable>,
    mut ev_block: EventReader<SelectionBlockEvent>
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return
    }

    let (cam, cam_transform) = cam_q.single();
    let Some(mouse_pos) = window_q.single().cursor_position() else {
        return
    };
    let ray = cam.viewport_to_world(cam_transform, mouse_pos).unwrap();

    if let Some((target, _)) = ray_cast.cast_ray(ray, &RayCastSettings::default()).first() {
        if entity_selected.contains(*target) || ev_block.read().count() > 0 {
            return;
        }

        if let Ok(selectable) = selectable_q.get(*target) {
            entity_selected.set(Some(selectable.entity_type.clone()));
            commands.trigger(SelectionUpdateEvent);
            return;
        }
    }

    // If this is reached, means something other than a Selectable entity has been clicked. In other words, deselect.
    if entity_selected.is_some() {
        entity_selected.set(None);
        commands.trigger(SelectionUpdateEvent);
    }
}

/// System to update the Selectable components of each entity on selection updates. Triggers with SelectionUpdateEvent
fn update_selectables(
    _: Trigger<SelectionUpdateEvent>,
    structure: Res<Structure>,
    entity_selected: Res<EntitySelected>,
    mut selectable_query: Query<&mut Selectable>,
    joint_q: Query<&Joint>,
) {
    info!(":: SELECTED: {:?}", entity_selected.0);

    for mut selectable in selectable_query.iter_mut() {
        selectable.selected = false;
    }

    if !entity_selected.is_some() {
        return;
    }

    let entity = entity_selected.get().unwrap();
    let mut selectable = selectable_query.get_mut(entity).unwrap();
    selectable.selected = true;

    if entity_selected.is_joint() {
        update_parent_edge(entity, &structure, &mut selectable_query, &joint_q, true);
    }
}

/// System to change the material of selected entities (via the Selectable component).
fn highlight_selection(
    select_materials: Res<HighlightMaterials>,
    joint_materials: Res<JointMaterial>,
    mut selectable_q: Query<(&mut MeshMaterial3d<StandardMaterial>, &Selectable), Changed<Selectable>>,
) {
    for (mut material_handle, selectable) in selectable_q.iter_mut() {
        if selectable.selected {
            match selectable.entity_type {
                SelectableEntity::Joint(_) => *material_handle = MeshMaterial3d(select_materials.joint_color.clone()),
                SelectableEntity::Connector(_) => *material_handle = MeshMaterial3d(select_materials.connector_color.clone()),
                SelectableEntity::Muscle(_) => *material_handle = MeshMaterial3d(select_materials.muscle_color.clone()),
            }
        } else {
            match selectable.entity_type {
                SelectableEntity::Joint(_) => *material_handle = MeshMaterial3d(joint_materials.joint_color.clone()),
                SelectableEntity::Connector(_) => *material_handle = MeshMaterial3d(joint_materials.connector_color.clone()),
                SelectableEntity::Muscle(_) => *material_handle = MeshMaterial3d(joint_materials.muscle_color.clone())
            }
        }
    }
}

// Changes the selected status of the connecting edge to the parent joint.
fn update_parent_edge(
    entity: Entity,
    structure: &Res<Structure>,
    selectable_q: &mut Query<&mut Selectable>,
    joint_q: &Query<&Joint>,
    selected: bool,
) {
    let Ok(joint_data) = joint_q.get(entity) else {
        return
    };

    let pdata = structure.node_weight(joint_data.node_index).unwrap();
    if let Some(parent) = pdata.parent {
        let Some(edge) = structure.find_edge(joint_data.node_index, parent) else {
            warn!("Edge missing between node {:?} and parent {:?}", joint_data.node_index, parent);
            return;
        };
        let conn = structure.edge_to_entity(edge).unwrap();
        let mut selectable: Mut<'_, Selectable> = selectable_q.get_mut(conn).unwrap();
        selectable.selected = selected;
    }
}