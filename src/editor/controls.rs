use bevy::prelude::*;

use super::{delete, save, UnitAxis};
use crate::selection::{EntitySelected, SelectableEntity};

#[derive(Event)]
pub enum ActionEvent {
    Confirm,
    Cancel,
    Delete,
    Save,

    JointAdd,
    JointLink,
    MuscleAdd,
    AdjustGrab,
    AdjustExtend,
    AdjustRotate,

    AxisChange(UnitAxis),
}

#[derive(Event)]
pub enum CursorControlEvent {
    Position(Vec2),
    Visible(bool),
    GrabMode(bevy::window::CursorGrabMode),
}

#[derive(Event)]
pub struct UndoEvent(Entity);

#[derive(Event)]
pub struct CacheEvent(Entity);


#[derive(Default, Debug, Clone, Copy, Resource)]
pub enum EditMode {
    #[default]
    Default,
    JointAdd(Entity),
    JointLink(Entity),
    MuscleAdd(Entity),
    AdjustGrab(Entity),
    AdjustExtend(Entity),
    AdjustAxis(Entity, UnitAxis),
    AdjustRotate(Entity),
    AdjustRotateAxis(Entity, UnitAxis),
}

#[derive(Resource)]
pub struct EditorControls {
    pub joint_add_key: Option<KeyCode>,
    pub joint_link_key: Option<KeyCode>,
    pub muscle_add_key: Option<KeyCode>,
    pub adjust_grab_key: Option<KeyCode>,
    pub adjust_rotate_key: Option<KeyCode>,
    pub adjust_extend_key: Option<KeyCode>,
    pub axis_x_key: Option<KeyCode>,
    pub axis_y_key: Option<KeyCode>,
    pub axis_z_key: Option<KeyCode>,
    pub action_confirm_key: Option<MouseButton>,
    pub action_cancel_key: Option<KeyCode>,
    pub action_delete_key: Option<KeyCode>,
    pub action_save_key: Option<KeyCode>,
}

impl Default for EditorControls {
    fn default() -> Self {
        EditorControls {
            joint_add_key: Some(KeyCode::Tab),
            joint_link_key: Some(KeyCode::KeyL),
            muscle_add_key: Some(KeyCode::KeyM),
            adjust_grab_key: Some(KeyCode::KeyG),
            adjust_rotate_key: Some(KeyCode::KeyR),
            adjust_extend_key: Some(KeyCode::KeyE),
            axis_x_key: Some(KeyCode::KeyX),
            axis_y_key: Some(KeyCode::KeyY),
            axis_z_key: Some(KeyCode::KeyZ),
            action_confirm_key: Some(MouseButton::Left),
            action_cancel_key: Some(KeyCode::Escape),
            action_delete_key: Some(KeyCode::Delete),
            action_save_key: Some(KeyCode::KeyS),
        }
    }
}

impl EditorControls {
    pub fn key_to_action(
        &self,
        key: KeyCode
    ) -> Option<ActionEvent> {
        match key {
            _ if Some(key) == self.joint_add_key => Some(ActionEvent::JointAdd),
            _ if Some(key) == self.joint_link_key => Some(ActionEvent::JointLink),
            _ if Some(key) == self.muscle_add_key => Some(ActionEvent::MuscleAdd),
            _ if Some(key) == self.adjust_grab_key => Some(ActionEvent::AdjustGrab),
            _ if Some(key) == self.adjust_rotate_key => Some(ActionEvent::AdjustRotate),
            _ if Some(key) == self.adjust_extend_key => Some(ActionEvent::AdjustExtend),
            _ if Some(key) == self.axis_x_key => Some(ActionEvent::AxisChange(UnitAxis::X)),
            _ if Some(key) == self.axis_y_key => Some(ActionEvent::AxisChange(UnitAxis::Y)),
            _ if Some(key) == self.axis_z_key => Some(ActionEvent::AxisChange(UnitAxis::Z)),
            // _ if Some(key) == self.action_confirm_key => Some(ActionEvent::Confirm),
            _ if Some(key) == self.action_cancel_key => Some(ActionEvent::Cancel),
            _ if Some(key) == self.action_delete_key => Some(ActionEvent::Delete),
            _ if Some(key) == self.action_save_key => Some(ActionEvent::Save),
            _ => None,
        }
    }

    pub fn mouse_to_action(
        &self,
        button: MouseButton
    ) -> Option<ActionEvent> {
        match button {
            _ if Some(button) == self.action_confirm_key => Some(ActionEvent::Confirm),
            _ => None,
        }
    }
}

pub fn input_to_actions(
    mut ev_action: EventWriter<ActionEvent>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    controls: Res<EditorControls>,
) {
    for key in key_input.get_just_pressed() {
        if let Some(action) = controls.key_to_action(*key) {
            ev_action.send(action);
        }
    }

    for button in mouse_input.get_just_pressed() {
        if let Some(action) = controls.mouse_to_action(*button) {
            ev_action.send(action);
        }
    }
}

pub fn editor_control(
    mut commands: Commands,
    mut edit_mode: ResMut<EditMode>,
    entity_selected: Res<EntitySelected>,
    mut ev_action: EventReader<ActionEvent>,
    mut ev_cursor: EventReader<CursorControlEvent>,
    mut window_q: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    for action in ev_action.read() {
        match *edit_mode {
            EditMode::Default => {
                if let Some(SelectableEntity::Joint(joint)) = entity_selected.0 {
                    match action {
                        ActionEvent::JointAdd => {
                            *edit_mode = EditMode::JointAdd(joint);
                        },
                        ActionEvent::JointLink => {
                            *edit_mode = EditMode::JointLink(joint);
                        }
                        ActionEvent::AdjustExtend => {
                            commands.send_event(CursorControlEvent::GrabMode(bevy::window::CursorGrabMode::Confined));

                            commands.send_event(CacheEvent(joint));
                            *edit_mode = EditMode::AdjustExtend(joint);
                        },
                        ActionEvent::AdjustGrab => {
                            commands.send_event(CacheEvent(joint));
                            *edit_mode = EditMode::AdjustGrab(joint);
                        },
                        ActionEvent::AdjustRotate => {
                            commands.send_event(CacheEvent(joint));
                            *edit_mode = EditMode::AdjustRotate(joint);
                        },
                        _ => (),
                    }
                } else if let Some(SelectableEntity::Connector(connector)) = entity_selected.0 {
                    match action {
                        ActionEvent::MuscleAdd => {
                            *edit_mode = EditMode::MuscleAdd(connector);
                        },
                        _ => (),
                    }
                }
            },
            EditMode::JointAdd(e) => {
                match action {
                    ActionEvent::JointAdd | ActionEvent::Cancel => *edit_mode = EditMode::Default,
                    ActionEvent::Confirm => {
                        if entity_selected.contains(e) {
                            commands.send_event(super::joint::JointAddEvent); 
                        } else {
                            *edit_mode = EditMode::Default;
                        }
                    },
                    _ => (),
                }
            },
            EditMode::JointLink(e) => {
                match action {
                    ActionEvent::Cancel => *edit_mode = EditMode::Default,
                    ActionEvent::Confirm => {
                        if entity_selected.is_joint() && !entity_selected.contains(e) {
                            commands.send_event(super::joint::JointLinkEvent); 
                        } else {
                            *edit_mode = EditMode::Default;
                        }
                    },
                    _ => (),
                }
            },
            EditMode::MuscleAdd(e) => {
                match action {
                    ActionEvent::Cancel => *edit_mode = EditMode::Default,
                    ActionEvent::Confirm => {
                        if entity_selected.is_connector() && !entity_selected.contains(e) {
                            commands.send_event(super::muscle::MuscleAddEvent); 
                        } else {
                            *edit_mode = EditMode::Default;
                        }
                    },
                    _ => (),
                }
            },
            EditMode::AdjustGrab(e) => {
                match action {
                    ActionEvent::AdjustGrab | ActionEvent::Cancel => {
                        commands.send_event(UndoEvent(e));
                        *edit_mode = EditMode::Default
                    },
                    ActionEvent::Confirm => {
                        *edit_mode = EditMode::Default;
                    },
                    ActionEvent::AxisChange(axis) => {
                        commands.send_event(UndoEvent(e));
                        *edit_mode = EditMode::AdjustAxis(e, *axis);
                    }
                    _ => (),
                }
            },
            EditMode::AdjustExtend(e) => {
                match action {
                    ActionEvent::AdjustExtend | ActionEvent::Cancel => {
                        commands.send_event(CursorControlEvent::GrabMode(bevy::window::CursorGrabMode::None));

                        commands.send_event(UndoEvent(e));
                        *edit_mode = EditMode::Default
                    },
                    ActionEvent::Confirm => {
                        commands.send_event(CursorControlEvent::GrabMode(bevy::window::CursorGrabMode::None));

                        *edit_mode = EditMode::Default;
                    },
                    _ => (),
                }
            },
            EditMode::AdjustAxis(e, axis) => {
                match action {
                    ActionEvent::AxisChange(new_axis) => {
                        commands.send_event(UndoEvent(e));

                        if *new_axis == axis {
                            *edit_mode = EditMode::AdjustGrab(e);
                        } else {
                            *edit_mode = EditMode::AdjustAxis(e, *new_axis);
                        }
                    },
                    ActionEvent::Cancel => {
                        commands.send_event(CursorControlEvent::GrabMode(bevy::window::CursorGrabMode::None));

                        commands.send_event(UndoEvent(e));
                        *edit_mode = EditMode::Default;
                    }
                    ActionEvent::Confirm => {
                        commands.send_event(CursorControlEvent::GrabMode(bevy::window::CursorGrabMode::None));

                        *edit_mode = EditMode::Default;
                    },
                    _ => (),
                }
            },
            EditMode::AdjustRotate(e) => {
                match action {
                    ActionEvent::AdjustRotate | ActionEvent::Cancel => {
                        commands.send_event(UndoEvent(e));
                        *edit_mode = EditMode::Default
                    },
                    ActionEvent::Confirm => {
                        *edit_mode = EditMode::Default;
                    },
                    ActionEvent::AxisChange(axis) => {
                        commands.send_event(UndoEvent(e));
                        *edit_mode = EditMode::AdjustRotateAxis(e, *axis);
                    }
                    _ => (),
                }
            },
            EditMode::AdjustRotateAxis(e, axis) => {
                match action {
                    ActionEvent::AxisChange(new_axis) => {
                        commands.send_event(UndoEvent(e));

                        if *new_axis == axis {
                            *edit_mode = EditMode::AdjustRotate(e);
                        } else {
                            *edit_mode = EditMode::AdjustRotateAxis(e, *new_axis);
                        }
                    },
                    ActionEvent::Cancel => {
                        commands.send_event(UndoEvent(e));
                        *edit_mode = EditMode::Default;
                    }
                    ActionEvent::Confirm => {
                        *edit_mode = EditMode::Default;
                    },
                    _ => (),
                }
            },
        }

        match action {
            ActionEvent::Save => { commands.trigger(save::SaveEvent); },
            ActionEvent::Delete => { commands.trigger(delete::DeleteEvent); },
            _ => (),
        }
    }

    if !ev_cursor.is_empty() {
        let Ok(mut window) = window_q.get_single_mut() else {
            error!("Failed to get primary window");
            return;
        };
        for event in ev_cursor.read() {
            match *event {
                CursorControlEvent::Position(pos) => {
                    window.set_cursor_position(Some(pos));
                },
                CursorControlEvent::Visible(visible) => {
                    window.cursor_options.visible = visible;
                },
                CursorControlEvent::GrabMode(mode) => {
                    window.cursor_options.grab_mode = mode;
                }
            }
        }
    }
}

pub fn undo(
    mut pos_cache: Local<Option<Vec3>>,
    mut ev_undo: EventReader<UndoEvent>,
    mut ev_cache: EventReader<CacheEvent>,
    mut transform_q: Query<&mut Transform>,
) {
    for event in ev_undo.read() {
        if let Some(position) = *pos_cache {
            let mut transform = transform_q.get_mut(event.0).unwrap();
            transform.translation = position;
        }
    }

    for event in ev_cache.read() {
        let transform = transform_q.get(event.0).unwrap();
        *pos_cache = Some(transform.translation);
    }
}