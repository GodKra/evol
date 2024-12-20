use bevy::prelude::*;
use crate::{selection::EntitySelected, Editor, GameState};


pub struct EditorUiPlugin;
impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Editor), 
            init
        )
        .add_systems(
            Update,
            (
                update_pos_info,
                tbutton_interact
            ).run_if(in_state(GameState::Editor))
        );
    }
}

#[derive(Component)]
struct PosText;

#[derive(Component)]
struct TButton;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.35, 0.35);

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font_handle: Handle<Font> = asset_server.load("fonts\\FiraCode-Regular.ttf");
    commands.spawn((
        Text::default(),
        TextFont {
            font: font_handle.clone(),
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
        PosText,
        Editor
    ));

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Editor
    )).with_children(|parent| {
        parent.spawn((
            Button,
            Node {
                width: Val::Px(80.0),
                height: Val::Px(30.0),
                margin: UiRect {
                    top: Val::Px(5.),
                    ..default()
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            TButton,
        )).with_child((
            Text::new("TEST"),
            TextFont {
                font: font_handle.clone(),
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
    });
}


/// System to update top left coordinate/position information.
fn update_pos_info(
    entity_selected: Res<EntitySelected>,
    transform_q: Query<&Transform>,
    mut text_q: Query<&mut Text, With<PosText>>,
) {
    let mut text = text_q.single_mut();

    match entity_selected.get() {
        Some(joint) => {
            let transform = transform_q.get(joint).unwrap();

            **text = format!(
                "X: {:.3} | Y: {:.3} | Z: {:.3}",
                transform.translation.x, transform.translation.y, transform.translation.z
            );
        },
        None => {
            **text = "".to_string();
        },
     }
}

fn tbutton_interact(
    mut state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<TButton>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                println!("Switching State to GameState::Observer");
                state.set(GameState::Observer)
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