use bevy::prelude::*;
use bevy_mod_picking::PickingCameraBundle;
// use bevy_mod_picking::PickingCameraBundle;
use iyes_loopless::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{pgraph::PGraph, util::*};

mod joint;
mod muscle;

pub struct ObserverPlugin;
impl Plugin for ObserverPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_enter_system(crate::GameState::Observer, setup_graphics)
        .add_enter_system(crate::GameState::Observer, setup_physics)
        .add_enter_system(crate::GameState::Observer, deserialize_pgraph)
        .add_enter_system(crate::GameState::Observer, setup_joint_physics.after(deserialize_pgraph))
        .add_system(
            joint::update_connector
            .run_in_state(crate::GameState::Observer)
        )
        .add_system(
            muscle::update_muscles
            .run_in_state(crate::GameState::Observer)
        )
        ;
    }
}

fn deserialize_pgraph(
    mut commands: Commands,
    mut graph: ResMut<PGraph>,
    meshes: Res<JointMeshes>,
    materials: Res<JointMaterial>,
) {
    let graph_data = &std::fs::read("./pgraph.ron").unwrap();
    graph.0 = ron::de::from_bytes(graph_data).unwrap();
    println!("** GENERATED GRAPH");

    let extras = (
        RigidBody::Dynamic,
        Collider::ball(1.0),
        Friction::coefficient(3.0) ,
        GravityScale(2.0)
    );

    graph.create(&mut commands, meshes, materials, extras, crate::Observer);
}

fn setup_joint_physics(
    pgraph: ResMut<PGraph>,
    mut commands: Commands,
) {
    for edge in pgraph.0.edge_indices() {
        let (n1, n2) = pgraph.0.edge_endpoints(edge).unwrap();
        let n1_weight = pgraph.0.node_weight(n1).unwrap();
        let n2_weight = pgraph.0.node_weight(n2).unwrap();

        let s_joint = SphericalJointBuilder::new().local_anchor2(n2_weight.pos-n1_weight.pos);

        commands.entity(n1_weight.entityid.unwrap()).insert(ImpulseJoint::new(n2_weight.entityid.unwrap(), s_joint));
    }
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 10.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        PickingCameraBundle::default(),
        crate::camera::PanOrbitCamera {
            radius: 40.,
            focus: Vec3::ZERO,
            ..Default::default()
        },
        crate::Observer
    ));

    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        },
        crate::Observer
    ));
}

fn setup_physics( 
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /* Create the ground. */
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1000.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0., -5., 0.),
            ..default()
        },
        Collider::cuboid(100.0, 0.1, 100.0),
        Restitution::coefficient(0.0),
        Friction::coefficient(3.0),
        crate::Observer,
    ));

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