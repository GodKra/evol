use bevy::prelude::*;
// use bevy_mod_picking::PickingCameraBundle;
use iyes_loopless::prelude::*;
use bevy_rapier3d::prelude::*;

mod joint;

pub struct ObserverPlugin;
impl Plugin for ObserverPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_enter_system(crate::GameState::Observer, setup_graphics)
        .add_enter_system(crate::GameState::Observer, setup_physics)
        .add_enter_system(crate::GameState::Observer, joint::generate_mesh)
        .add_system(joint::update_connector.run_in_state(crate::GameState::Observer))
        ;
    }
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 10.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
        .insert(crate::camera::PanOrbitCamera {
            radius: 40.,
            focus: Vec3::ZERO,
            ..Default::default()
        })
        .insert(crate::Observer);

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    }).insert(crate::Observer);
}

fn setup_physics( 
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /* Create the ground. */
    commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1000.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0., -10., 0.),
            ..default()
        })
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(Restitution::coefficient(0.0))
        .insert(Friction::coefficient(3.0))
        // .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)))
        .insert(crate::Observer);

    /* Create the bouncing ball. */
    // commands.spawn_bundle(PbrBundle {
    //         mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //         material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    //         transform: Transform::from_xyz(0.0, 5.0, 0.0),
    //         ..default()
    //     })
    //     .insert(RigidBody::Dynamic)
    //     .insert(Collider::ball(0.5))
    //     .insert(Restitution::coefficient(1.0))
    //     // .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)))
    //     .insert(crate::Observer);
        
}



// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     println!("here");
//     commands.spawn_bundle(PbrBundle {
//         mesh: meshes.add(Mesh::from(shape::Plane { size: 1000.0 })),
//         material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
//         ..default()
//     }).insert(crate::Observer);

//     commands.spawn_bundle(PointLightBundle {
//         point_light: PointLight {
//             intensity: 1500.0,
//             shadows_enabled: true,
//             ..default()
//         },
//         transform: Transform::from_xyz(4.0, 8.0, 4.0),
//         ..default()
//     }).insert(crate::Observer);

//     commands.spawn_bundle(PbrBundle {
//         mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//         material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
//         transform: Transform::from_xyz(0.0, 0.5, 0.0),
//         ..default()
//     }).insert(crate::Observer);

//     // Camera
//     let translation = Vec3::new(0.0, 0.0, 10.0);
//     let radius = translation.length();

//     // let mut camera = OrthographicCameraBundle::new_3d();
//     // camera.orthographic_projection.scale = 3.0;
//     // camera.transform = Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y);
//     commands.spawn_bundle(PerspectiveCameraBundle {
//         transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
//         ..Default::default()
//     }).insert_bundle(PickingCameraBundle::default())
//         .insert(crate::camera::PanOrbitCamera {
//             radius,
//             ..Default::default()
//         })
//         .insert(crate::Observer);
//     // commands.insert_resource(AmbientLight {
//     //     color: Color::WHITE,
//     //     brightness: 0.3,
//     // });
//     // Background color
//     commands.insert_resource(
//         ClearColor(
//             Color::rgb(0.4, 0.4, 0.4)
//         )
//     );
// }