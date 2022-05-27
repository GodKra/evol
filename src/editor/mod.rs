pub mod controls;
pub mod cursor;
pub mod grab;
pub mod save;
pub mod delete;
pub mod selection;
pub mod ui;

use bevy::{prelude::*};
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

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IsAdjustMode>()
            .init_resource::<PositionCache>()
            .init_resource::<MovementCache>()
            .add_plugin(ui::EditorUiPlugin)
            .add_plugin(selection::SelectionPlugin)
            .add_system(
                self::grab::grab_control
                .label(GRAB_CTRL)
                .before(JOINT_SELECT))
            .add_system(
                self::cursor::cursor_control
                    .label(CRSR_CTRL)
                    .after(GRAB_CTRL))
            .add_system(
                self::controls::editor_mode_toggle
                    .label(MODE_TOGGLE)
                    .after(CRSR_CTRL)
                    .after(GRAB_CTRL))
            .add_system(
                self::grab::update_connector
                    .after(MODE_TOGGLE))
            
            .add_system(
                self::save::save
            )
            .add_system(
                self::delete::delete_joint
            );
    }
}

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