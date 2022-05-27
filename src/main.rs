use bevy::{prelude::*};
use bevy_mod_picking::*;

mod points;
mod camera;
mod editor;
mod util;

use points::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(WindowDescriptor {
            title: "test".to_string(),
            // mode: bevy::window::WindowMode::SizedFullscreen,
            width: 700.,
            height: 700.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        // .add_plugin(DebugCursorPickingPlugin)
        .add_plugin(camera::PanOrbitCameraPlugin)
        .add_plugin(editor::EditorPlugin)
        .init_resource::<JointMeshes>()
        .init_resource::<JointMaterial>()
        .add_startup_system(setup)
        .add_startup_system(generate_mesh)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    // Camera
    let translation = Vec3::new(0.0, 0.0, 10.0);
    let radius = translation.length();

    // let mut camera = OrthographicCameraBundle::new_3d();
    // camera.orthographic_projection.scale = 3.0;
    // camera.transform = Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(translation)
        .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    }).insert_bundle(PickingCameraBundle::default())
        .insert(camera::PanOrbitCamera {
            radius,
            ..Default::default()
        });
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
    });
    // Background color
    commands.insert_resource(
        ClearColor(
            Color::rgb(0.4, 0.4, 0.4)
        )
    );
}

