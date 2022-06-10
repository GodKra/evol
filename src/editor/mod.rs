pub mod controls;
pub mod cursor;
pub mod grab;
pub mod save;
pub mod selection;
pub mod ui;

use bevy::{prelude::*};
use bevy_mod_picking::PickingCameraBundle;
use iyes_loopless::prelude::*;
use crate::util::*;

use self::selection::*;

/* EDITOR SYSTEM ORDER
UPDATE STAGE: grab_ctrl -> crsr_ctrl -> mode_toggle -> joint_select -> update_pos_info
                                                    -> update_connector
            save, delete_joint
     \/
MANAGE_SELECT STAGE: selection_type_update -> selection_highlight
*/


pub const CRSR_CTRL: &str = "cursor_control";
pub const GRAB_CTRL: &str = "grab_control";
pub const MODE_TOGGLE: &str = "edit_mode_toggle";

//
// Data types
//

#[derive(Debug, PartialEq)]
pub enum EditMode {
    Cursor,
    GrabFull,    GrabExtend,
    GrabAxis(Axis),
    RotateFull,
    // RotateAxis(Axis),
}

#[derive(Debug, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
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

#[derive(Default)]
pub struct IsAdjustMode(bool);

#[derive(Default)]
pub struct PositionCache(Vec3);

#[derive(Default)]
pub struct MovementCache(f32);

//
// Components
//

#[derive(Default, Component)]
pub struct EditCursor;

#[derive(Default, Component, Debug)]
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
            .init_resource::<PositionCache>()
            .init_resource::<MovementCache>()
            .add_plugin(ui::EditorUiPlugin)
            .add_plugin(selection::SelectionPlugin)
            // .add_system_set(
            //     SystemSet::on_update(crate::AppState::Editor)
            //         .with_system(
            //             self::grab::grab_control
            //             .label(GRAB_CTRL)
            //             .before(JOINT_SELECT))
            //         .with_system(
            //             self::cursor::cursor_control
            //                 .label(CRSR_CTRL)
            //                 .after(GRAB_CTRL))
            //         .with_system(
            //             self::controls::editor_mode_toggle
            //                 .label(MODE_TOGGLE)
            //                 .after(CRSR_CTRL)
            //                 .after(GRAB_CTRL))
            //         .with_system(
            //             self::grab::update_connector
            //                 .after(MODE_TOGGLE))
                    
            //         .with_system(
            //             self::save::save
            //         )
            //         .with_system(
            //             self::delete::delete_joint
            //         )
            //         .into()
            // );
            .add_enter_system(crate::GameState::Editor, setup)
            .add_system(
                self::grab::grab_control
                .run_in_state(crate::GameState::Editor)
                .label(GRAB_CTRL)
                .before(JOINT_SELECT))
            .add_system(
                self::cursor::cursor_control
                    .run_in_state(crate::GameState::Editor)
                    .label(CRSR_CTRL)
                    .after(GRAB_CTRL))
            .add_system(
                self::controls::editor_mode_toggle
                    .run_in_state(crate::GameState::Editor)
                    .label(MODE_TOGGLE)
                    .after(CRSR_CTRL)
                    .after(GRAB_CTRL))
            .add_system(
                self::grab::update_connector
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
                self::save::delete_joint
                .run_in_state(crate::GameState::Editor)
                .run_if(|input: Res<Input<KeyCode>>| {
                    KeyControls::EDELETE.pressed(input)
                })
            );
    }
}

fn setup(
    mut commands: Commands,
) {
    // Camera
    let translation = Vec3::new(0.0, 0.0, 10.0);
    let radius = translation.length();

    // let mut camera = OrthographicCameraBundle::new_3d();
    // camera.orthographic_projection.scale = 3.0;
    // camera.transform = Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(translation)
        .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    }).insert_bundle(PickingCameraBundle::default())
        .insert(crate::camera::PanOrbitCamera {
            radius,
            ..Default::default()
        });
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