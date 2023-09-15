use bevy::prelude::*;
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
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "test".to_string(),
                    resolution: (700., 700.,).into(),
                    ..default()
                }),
                ..default()
            }),
            DefaultPickingPlugins,
            camera::PanOrbitCameraPlugin,
            selection::SelectionPlugin,
            editor::EditorPlugin,
            observer::ObserverPlugin,
        ))
        .init_resource::<util::JointMeshes>()
        .init_resource::<util::JointMaterial>()
        .run();
}