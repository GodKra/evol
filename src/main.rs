use bevy::{prelude::*};
use bevy_mod_picking::*;

use iyes_loopless::prelude::*;

mod camera;
mod editor;
mod observer;
mod util;


#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Editor,
    Observer,
}

/// Marker component for entities belonging to the Editor state. All marked with this will be despawned on state change.
#[derive(Component)]
pub struct Editor;
/// Marker component for entities belonging to the Observer state. All marked with this will be despawned on state change.
#[derive(Component)]
pub struct Observer;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        // Set WindowDescriptor Resource to change title and size
        // .insert_resource(WindowDescriptor {
        //     title: "test".to_string(),
        //     // mode: bevy::window::WindowMode::SizedFullscreen,
        //     width: 700.,
        //     height: 700.,
        //     ..Default::default()
        // })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                    title: "test".to_string(),
                    // mode: bevy::window::WindowMode::SizedFullscreen,
                    width: 700.,
                    height: 700.,
                    ..default()
                },
            ..default()
        }))
        .add_loopless_state(GameState::Editor)
        .add_plugin(PickingPlugin)
        // .add_plugin(DebugCursorPickingPlugin)
        .add_plugin(camera::PanOrbitCameraPlugin)
        .add_plugin(editor::EditorPlugin)
        // .add_plugin(observer::ObserverPlugin)
        .init_resource::<util::JointMeshes>()
        .init_resource::<util::JointMaterial>()
        .add_exit_system(GameState::Editor, util::despawn_with::<Editor>)
        .add_enter_system(GameState::Observer, test)
        .add_system(testbut_interact.run_in_state(GameState::Observer))
        .add_exit_system(GameState::Observer, util::despawn_with::<Observer>)
        .run();
}

#[derive(Component)]
struct TestBut;

fn test(
    mut commands: Commands,
    asset_server: Res<AssetServer>, 
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            ..default()
        }).with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(80.0), Val::Px(30.0)),
                        margin: UiRect {
                            top: Val::Px(5.),
                            ..default()
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "GO",
                            TextStyle {
                                font: asset_server.load("fonts/FiraCode-Regular.ttf"),
                                font_size: 15.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ),
                        ..default()
                    });
                })
                .insert(TestBut);
        }).insert(Observer);
}


fn testbut_interact(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<TestBut>),
    >,
) {
    for (interaction, _) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                println!("Switching State to GameState::Editor");
                commands.insert_resource(NextState(GameState::Editor));
            }
            Interaction::Hovered => {
            }
            Interaction::None => {
            }
        }
    }
}