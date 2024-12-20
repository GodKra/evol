use bevy::prelude::*;
use selection::SelectionPlugin;

mod camera;
mod editor;
mod selection;
// mod observer;
mod structure;
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
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "test".to_string(),
                    resolution: (700., 700.,).into(),
                    ..default()
                }),
                ..default()
            }),
            MeshPickingPlugin,
            SelectionPlugin,
            editor::EditorPlugin,
        ))
        .init_state::<GameState>()
        .init_resource::<util::JointMeshes>()
        .init_resource::<util::JointMaterial>()
        .run();
}