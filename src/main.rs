use bevy::{prelude::*};
use bevy_mod_picking::*;

mod camera;
mod editor;
mod selection;
mod observer;
mod pgraph;
mod util;

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Editor,
    Observer,
}

/// Marker component for entities belonging to the Editor state. All marked with this will be despawned on state change.
#[derive(Component, Clone, Copy)]
pub struct Editor;
/// Marker component for entities belonging to the Observer state. All marked with this will be despawned on state change.
#[derive(Component, Clone, Copy)]
pub struct Observer;

fn main() {
    App::new()
        .add_state::<GameState>()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "test".to_string(),
                resolution: (700., 700.,).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(PickingPlugin)
        .add_plugin(camera::PanOrbitCameraPlugin)
        .add_plugin(selection::SelectionPlugin)
        .add_plugin(editor::EditorPlugin)
        .add_plugin(observer::ObserverPlugin)
        .init_resource::<util::JointMeshes>()
        .init_resource::<util::JointMaterial>()
        .add_system(test.in_schedule(OnEnter(GameState::Observer)))
        .add_system(testbut_interact.in_set(OnUpdate(GameState::Observer)))
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
    // mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<TestBut>),
    >,
) {
    for (interaction, _) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                println!("Switching State to GameState::Editor");
                state.set(GameState::Editor);
            }
            Interaction::Hovered => {
            }
            Interaction::None => {
            }
        }
    }
}