pub mod controls;
pub mod cursor;
pub mod adjust;
pub mod joint;
pub mod muscle;
pub mod save;
pub mod delete;
pub mod ui;

use bevy::{prelude::*};
use bevy_mod_picking::PickingCameraBundle;
use iyes_loopless::prelude::*;

use crate::editor::joint::LinkRoot;
use crate::editor::muscle::MuscleRoot;
use crate::pgraph::PGraph;
use crate::{util::*};

use crate::selection::*;

/* EDITOR SYSTEM ORDER
UPDATE STAGE: adjst_ctrl -> crsr_ctrl -> mode_toggle -> joint_select     ->  muscle_construct -> update_muscles
                                                       update_connector
                                                       update_pgraph_pos 
            save, delete_joint
     \/
MANAGE_SELECT STAGE: selection_type_update -> selection_highlight
*/


pub const CRSR_CTRL: &str = "cursor_control";
pub const ADJST_CTRL: &str = "adjust_control";
pub const MODE_TOGGLE: &str = "edit_mode_toggle";
pub const MUSCLE_CONSTRUCT: &str = "muscle_construct";

//
// Assets
//

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EditMode {
    Cursor,
    GrabFull,
    GrabExtend,
    GrabAxis(PosAxis),
    RotateFull,
    RotateAxis(PosAxis),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PosAxis {
    X,
    Y,
    Z,
}

impl PosAxis {
    pub fn to_vec(&self) -> Vec3 {
        match self {
            Self::X => Vec3::X,
            Self::Y => Vec3::Y,
            Self::Z => Vec3::Z,
        }
    }

    pub fn from_key(key: &KeyCode) -> Option<Self> {
        match key {
            KeyCode::X => Some(Self::X),
            KeyCode::Y => Some(Self::Y),
            KeyCode::Z => Some(Self::Z),
            _ => None,
        }
    } 
}

#[derive(Default, Resource)]
pub struct IsAdjustMode(bool);


#[derive(Default, Resource)]
pub struct IsMuscleMode(bool);

#[derive(Default, Resource)]
pub struct IsLinkMode(bool);

/// Stores the former position of a joint when in Grab mode.
#[derive(Default, Resource)]
pub struct PositionCache(Vec3);

/// Stores the total movement of a joint when in grab extrude/axis mode.
#[derive(Default, Resource)]
pub struct MovementCache(f32);

//
// Components
//

#[derive(Default, Component)]
pub struct EditCursor;

#[derive(Default, Component, Debug, Clone, Copy)]
pub struct Editable {
    pub mode: Option<EditMode>,
}

//
// Plugin
//

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IsAdjustMode>()
            .init_resource::<IsMuscleMode>()
            .init_resource::<IsLinkMode>()
            .init_resource::<PositionCache>()
            .init_resource::<MovementCache>()
            .init_resource::<PGraph>()
            .init_resource::<MuscleRoot>()
            .init_resource::<LinkRoot>()
            .add_plugin(ui::EditorUiPlugin)

            .add_enter_system(crate::GameState::Editor, setup)
            .add_enter_system(crate::GameState::Editor, deserialize_pgraph)

            .add_system(
                self::adjust::adjust_control
                .run_in_state(crate::GameState::Editor)
                .label(ADJST_CTRL)
                .before(JOINT_SELECT))
            .add_system(
                self::cursor::cursor_control
                    .run_in_state(crate::GameState::Editor)
                    .label(CRSR_CTRL)
                    .after(ADJST_CTRL))
            .add_system(
                self::controls::editor_mode_toggle
                    .run_in_state(crate::GameState::Editor)
                    .label(MODE_TOGGLE)
                    .after(CRSR_CTRL)
                    .after(ADJST_CTRL))
            .add_system(
                self::joint::update_connector
                    .run_in_state(crate::GameState::Editor)
                    .after(MODE_TOGGLE))
            .add_system(
                self::joint::update_pgraph_pos
                    .run_in_state(crate::GameState::Editor)
                    .after(MODE_TOGGLE))
            
            .add_system(
                self::save::save
                .run_in_state(crate::GameState::Editor)
                .run_if(|input: Res<Input<KeyCode>>| {
                    KeyControls::ESAVE.pressed(input)
                })
            )
            .add_system(
                self::delete::delete
                .run_in_state(crate::GameState::Editor)
                .run_if(|input: Res<Input<KeyCode>>| {
                    KeyControls::EDELETE.pressed(input)
                })
            )
            .add_system(
                muscle::muscle_construct
                .run_in_state(crate::GameState::Editor)
                .label(MUSCLE_CONSTRUCT)
                .after(JOINT_SELECT)
            )
            .add_system_to_stage(
                "manage_selection_stage", // this crashes in normal system order
                joint::link_joint
                .run_in_state(crate::GameState::Editor)
                // .after(JOINT_SELECT)
            )
            .add_system_to_stage(
                "manage_selection_stage", // because weird bug with muscles hanging when adjust is reset (FIX TODO)
                muscle::update_muscles
                .run_in_state(crate::GameState::Editor)
            )
            ;
            println!("done editor");
    }
}

fn deserialize_pgraph(
    mut commands: Commands,
    mut graph: ResMut<PGraph>,
    meshes: Res<JointMeshes>,
    materials: Res<JointMaterial>,
) {
    let graph_data = &std::fs::read("./pgraph.ron").unwrap();
    graph.0 = ron::de::from_bytes(graph_data).unwrap();
    println!("** GENERATED GRAPH");
    graph.create(&mut commands, meshes, materials, Editable { mode: None }, crate::Editor);
}

fn setup(
    mut commands: Commands,
) {
    // Camera
    let translation = Vec3::new(0.0, 0.0, 10.0);
    let radius = translation.length();

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation)
            .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        PickingCameraBundle::default(),
        crate::camera::PanOrbitCamera {
            radius,
            ..Default::default()
        },
        crate::Editor
    ));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
    });
    // Background color
    commands.insert_resource(
        ClearColor(
            Color::rgb(0.4, 0.4, 0.4)
        )
    );
}