use bevy::prelude::*;

use crate::points::*;

use super::*;

pub struct EditorUiPlugin;
impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init)
            .add_system(
                update_pos_info
                    .after(selection::JOINT_SELECT)
            );
    }
}

#[derive(Component)]
struct InfoText;


fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
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
    }).insert(InfoText);
}


/// System to update top left coordinate/position information.
/// 
/// *passive
fn update_pos_info(
    joint_selected: Res<JointSelected>,
    pos_cache: Res<PositionCache>,
    jointq: Query<(&Joint, &Transform, &GlobalTransform, &Editable)>,
    mut textq: Query<&mut Text, With<InfoText>>,
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