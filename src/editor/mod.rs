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

use crate::editor::joint::LinkRoot;
use crate::editor::muscle::MuscleRoot;
use crate::pgraph::PGraph;
use crate::{util::*, GameState};

use crate::selection::*;

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

            .add_systems(
                (setup, deserialize_pgraph).in_schedule(OnEnter(crate::GameState::Editor))
            )
            
            .add_systems(
                (
                    adjust::adjust_control,
                    cursor::cursor_control,
                    controls::editor_mode_toggle,
                    joint::update_connector,
                    joint::update_pgraph_pos,
                    joint::link_joint,
                    muscle::update_muscles,
                ).chain()
                 .in_set(OnUpdate(crate::GameState::Editor))
            )

            .add_systems(
                (
                    save::save
                        .run_if(|input: Res<Input<KeyCode>>| {
                            KeyControls::ESAVE.pressed(input)
                        }),
                    delete::delete
                        .run_if(|input: Res<Input<KeyCode>>| {
                            KeyControls::EDELETE.pressed(input)
                        }),
                    muscle::muscle_construct
                        .after(crate::selection::joint_select),
                ).in_set(OnUpdate(crate::GameState::Editor))
            )
            
            .add_system(crate::util::despawn_all::<crate::Editor>.in_schedule(OnExit(GameState::Editor)));
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