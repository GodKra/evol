use bevy::{prelude::*};

use crate::util::Errors;

use super::*;

/// System to handle editor mode toggle controls and set up the appropriate
/// resources
/// 
/// *active
pub fn editor_mode_toggle(
    entity_selected: Res<EntitySelected>,
    selection_updated: Res<SelectionUpdated>,
    mut is_adjust_mode: ResMut<IsAdjustMode>,
    mut pos_cache: ResMut<PositionCache>,
    mut mv_cache: ResMut<MovementCache>,
    key_input: Res<Input<KeyCode>>,
    mouse_btn: Res<Input<MouseButton>>,
    mut editable_q: Query<&mut Editable>,
    mut transform_q: Query<&mut Transform>,
) {
    if selection_updated.0 || !entity_selected.is_joint() {
        return;
    }

    let joint_selected = entity_selected.get().unwrap();
    let Ok(mut editable) = editable_q.get_mut(joint_selected) else {
        panic!("{}", Errors::ComponentMissing("Editable", joint_selected))
    };

    let key_inputs = key_input.get_just_pressed();

    for input in key_inputs {
        match input {
            KeyCode::Tab => {
                match &editable.mode {
                    Some(mode) => match mode {
                        EditMode::Cursor => editable.mode = None,
                        _ => (),
                    },
                    None => editable.mode = Some(EditMode::Cursor),
                }
            },
            KeyCode::R => {
                match &editable.mode {
                    Some(mode) => match mode {
                        _ => (),
                    },
                    None => {
                        is_adjust_mode.0 = true;
                        editable.mode = Some(EditMode::RotateFull);

                        let transform = transform_q.get(joint_selected).unwrap();
                        pos_cache.0 = transform.translation;
                    },
                }
            },
            KeyCode::G => {
                match &editable.mode {
                    None => {
                        is_adjust_mode.0 = true;
                        editable.mode = Some(EditMode::GrabFull);

                        let transform = transform_q.get(joint_selected).unwrap();
                        pos_cache.0 = transform.translation;
                    },
                    Some(mode) => match mode {
                        _ => (),

                    },
                }
            },
            KeyCode::E => {
                match &editable.mode {
                    None => {
                        is_adjust_mode.0 = true;
                        editable.mode = Some(EditMode::GrabExtend);

                        let transform = transform_q.get(joint_selected).unwrap();
                        pos_cache.0 = transform.translation;
                        mv_cache.0 = transform.translation.length();
                    }
                    _ => (),
                }
            },
            KeyCode::X | KeyCode::Y | KeyCode::Z => {
                match &editable.mode {
                    Some(mode) => {
                        let mut transform = transform_q.get_mut(joint_selected).unwrap();
                        transform.translation = pos_cache.0;

                        match mode {
                            EditMode::GrabFull | EditMode::GrabAxis(_) => {
                                editable.mode = Some(EditMode::GrabAxis(PosAxis::from_key(input).unwrap()))
                            },
                            EditMode::RotateFull | EditMode::RotateAxis(_) => {
                                editable.mode = Some(EditMode::RotateAxis(PosAxis::from_key(input).unwrap()))
                            }
                            _ => (),
                        }
                    },
                    None => (),
                }
            },
            KeyCode::Escape => {
                if is_adjust_mode.0 {
                    let mut transform = transform_q.get_mut(joint_selected).unwrap();
                    transform.translation = pos_cache.0;
                }

                is_adjust_mode.0 = false;
                editable.mode = None
            },
            _ => (),
        }
    }

    if mouse_btn.just_pressed(MouseButton::Left) {
        pos_cache.0 = Vec3::default();
        mv_cache.0 = 0.0;
        match &editable.mode {
            Some(_) => {
                is_adjust_mode.0 = false;
                editable.mode = None;
            },
            None => (),
        }
    }
}