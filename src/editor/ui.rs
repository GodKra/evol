use bevy::prelude::*;

use crate::points::*;

use super::*;

pub struct EditorUiPlugin;
impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(crate::GameState::Editor, init)
            .add_system(
                update_pos_info
                .run_in_state(crate::GameState::Editor)
                .after(selection::JOINT_SELECT)
            )
            .add_system(
                tbutton_interact
                .run_in_state(crate::GameState::Editor)
            );
        // app.add_startup_system(init)
        // .add_system_set(
        //     SystemSet::on_update(crate::AppState::Editor)
        //         .with_system(
        //             update_pos_info
        //                 .after(selection::JOINT_SELECT)
        //         )
        //         .with_system(tbutton_interact)
        // );
    }
}

#[derive(Component)]
struct PosText;

#[derive(Component)]
struct TButton;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.35, 0.35);

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Position information text
    commands.spawn_bundle(UiCameraBundle::default());
    let font = asset_server.load("fonts/FiraCode-Regular.ttf");
    commands.spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        },
        text: Text {
            sections: vec![
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 13.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 13.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 13.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 13.0,
                            color: Color::WHITE,
                        },
                    },
                ],
                alignment: Default::default(),
        },
        ..default()
    }).insert(PosText);

    // Transition button
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        }).with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(80.0), Val::Px(30.0)),
                        margin: Rect {
                            top: Val::Px(5.),
                            ..default()
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Test",
                            TextStyle {
                                font: asset_server.load("fonts/FiraCode-Regular.ttf"),
                                font_size: 15.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..default()
                    });
                })
                .insert(TButton);
        });
}


/// System to update top left coordinate/position information.
/// 
/// *passive
fn update_pos_info(
    joint_selected: Res<JointSelected>,
    pos_cache: Res<PositionCache>,
    jointq: Query<(&Joint, &Transform, &GlobalTransform, &Editable)>,
    mut textq: Query<&mut Text, With<PosText>>,
) {
    let mut text = textq.single_mut();
    match joint_selected.0 {
        Some(joint) => {
            let jq = jointq.get(joint);
            if jq.is_err() {
                // println!("UI: update_info | {:?}", jq); // not a problem
                return;
            }
            let (joint, ltransform, gtransform, editable) = jq.unwrap();

            if let Some(mode) = editable.mode.as_ref() {
                 match mode {
                    EditMode::Cursor => (),
                    _ => {
                        let dif = pos_cache.0 - ltransform.translation; // should store pos inside joint
                        text.sections[0].value = format!(
                            "dx: {:.3} | dy: {:.3} | dz: {:.3}  ({:.3})",
                            dif.x,
                            dif.y, 
                            dif.z,
                            dif.length(),
                        );
                        text.sections[1].value = "".to_string();
                        text.sections[2].value = "".to_string();
                        return;
                    },
                }
            }

            text.sections[0].value = format!(
                "Global  X: {:.3} | Y: {:.3} | Z: {:.3}",
                gtransform.translation.x, gtransform.translation.y, gtransform.translation.z
            );
            text.sections[1].value = format!(
                // Global -
                "\nLocal   X: {:.3} | Y: {:.3} | Z: {:.3}",
                ltransform.translation.x, ltransform.translation.y, ltransform.translation.z
            );
            text.sections[2].value = format!(
                "\nLength  {:.3}", joint.dist
            );
        },
        None => {
            text.sections[0].value = "".to_string();
            text.sections[1].value = "".to_string();
            text.sections[2].value = "".to_string();
        },
     }
}

fn tbutton_interact(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>, With<TButton>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}