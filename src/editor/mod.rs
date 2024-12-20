pub mod controls;
pub mod joint;
pub mod adjust;
pub mod muscle;
pub mod save;
pub mod delete;
pub mod ui;

use bevy::prelude::*;

use crate::structure::Structure;
use crate::util::{despawn_all, JointMaterial, JointMeshes};
use crate::GameState;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum UnitAxis {
    X,
    Y,
    Z,
}

impl UnitAxis {
    pub fn to_vec(self) -> Vec3 {
        match self {
            Self::X => Vec3::X,
            Self::Y => Vec3::Y,
            Self::Z => Vec3::Z,
        }
    }

    pub fn from_key(key: &KeyCode) -> Option<Self> {
        match key {
            KeyCode::KeyX => Some(Self::X),
            KeyCode::KeyY => Some(Self::Y),
            KeyCode::KeyZ => Some(Self::Z),
            _ => None,
        }
    } 
}

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Structure>()
            .init_resource::<controls::EditMode>()
            .init_resource::<controls::EditorControls>()

            .add_event::<controls::ActionEvent>()
            .add_event::<controls::CursorControlEvent>()
            .add_event::<controls::UndoEvent>()
            .add_event::<controls::CacheEvent>()
            .add_event::<joint::JointAddEvent>()
            .add_event::<joint::JointLinkEvent>()
            .add_event::<muscle::MuscleAddEvent>()
            .add_event::<save::SaveEvent>()
            .add_event::<delete::DeleteEvent>()

            .add_systems(
                OnEnter(GameState::Editor), 
                (deserialize_structure, setup)
            )

            .add_systems(
                PreUpdate, 
                (
                    controls::input_to_actions, 
                    controls::undo
                ).run_if(in_state(GameState::Editor))
            )

            .add_systems(
                Update, 
                (
                    crate::camera::pan_orbit_camera,
                    crate::camera::focus_selected_entity,

                    adjust::adjust_control,
                    joint::joint_add,
                    joint::joint_link,
                    muscle::muscle_construct,
                    muscle::update_muscles,
                ).run_if(in_state(GameState::Editor))
            )

            .add_systems(
                PostUpdate, 
                (
                    controls::editor_control,
                    joint::update_connector,
                    joint::update_structure_pos,
                ).run_if(in_state(GameState::Editor))
            )

            .add_observer(delete::delete)
            .add_observer(save::save)

            .add_systems(
                OnExit(GameState::Editor), 
                despawn_all::<crate::Editor>
            )
            
            .add_plugins(ui::EditorUiPlugin);

            
    }
}


fn deserialize_structure(
    mut commands: Commands,
    mut structure: ResMut<Structure>,
    meshes: Res<JointMeshes>,
    materials: Res<JointMaterial>,
) {
    let graph_data = &std::fs::read("./structure.ron").unwrap();
    structure.0 = ron::de::from_bytes(graph_data).unwrap();
    println!("** GENERATED GRAPH");
    structure.create(
        &mut commands, 
        meshes, 
        materials, 
        (),
        (), 
        (), 
        crate::Editor
    );
}


fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        crate::camera::PanOrbitCamera::default(),
        crate::Editor
    ));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 80.0,
    });
    // Background color
    commands.insert_resource(
        ClearColor(
            Color::srgb(0.4, 0.4, 0.4)
        )
    );
}