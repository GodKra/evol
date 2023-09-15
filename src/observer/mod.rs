use bevy::prelude::*;
use bevy_mod_picking::prelude::RaycastPickCamera;
// use bevy_mod_picking::PickingCameraBundle;
use bevy_rapier3d::prelude::*;
use petgraph::stable_graph::EdgeIndex;

use crate::{pgraph::PGraph, util::*, GameState};

mod joint;
mod muscle;

pub struct ObserverPlugin;
impl Plugin for ObserverPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(
            OnEnter(GameState::Observer),
            (
                setup_graphics,
                setup_physics,
                deserialize_pgraph,
                setup_joint_physics.after(deserialize_pgraph)
            )
        )
        .add_systems(
            Update,
            (
                muscle::clear_forces,
                (
                    joint::update_connector,
                    muscle::update_muscles,
                    muscle::activate_muscle,
                    muscle::contract_muscle,
                    pause_sim,
                ).after(muscle::clear_forces),
            ).run_if(in_state(GameState::Observer))
        )

        .add_systems(OnEnter(GameState::Observer), test)
        .add_systems(Update, testbut_interact.run_if(in_state(GameState::Observer)))

        .add_systems(OnExit(GameState::Observer), crate::util::despawn_all::<crate::Observer>);
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

    let joint_c = (
        RigidBody::Dynamic,
        Collider::ball(1.0),
        Friction::coefficient(3.0),
        ExternalForce::default(),
        GravityScale(2.0)
    );

    let muscle_c = (
        muscle::MuscleState::default(),
        muscle::MuscleLength::default(), 
    );
    let conn_c = (
        // Collider::ball(0.2),
        // RigidBody::Dynamic,
        // Friction::coefficient(3.0),
        // ExternalForce::default(),
        // GravityScale(2.0)
    );

    graph.create(
        &mut commands, 
        meshes, 
        materials, 
        joint_c, 
        conn_c,
        muscle_c,
        crate::Observer
    );
}

fn setup_joint_physics(
    pgraph: ResMut<PGraph>,
    mut commands: Commands,
) {
    // for edge in pgraph.0.edge_indices() {
    //     let (n1, n2) = pgraph.0.edge_endpoints(edge).unwrap();
    //     let n1_weight = pgraph.0.node_weight(n1).unwrap();
    //     let n2_weight = pgraph.0.node_weight(n2).unwrap();

    //     let s_joint: SphericalJointBuilder = SphericalJointBuilder::new().local_anchor2(n2_weight.pos-n1_weight.pos);

    //     commands.entity(n1_weight.entityid.unwrap()).insert(ImpulseJoint::new(n2_weight.entityid.unwrap(), s_joint));
    // }
    let p_edge: Option<EdgeIndex> = None;
    for edge in pgraph.0.edge_indices() {
        let edge_weight = pgraph.0.edge_weight(edge).unwrap();
        let (n1, n2) = pgraph.0.edge_endpoints(edge).unwrap();
        let n1_weight = pgraph.0.node_weight(n1).unwrap();
        let n2_weight = pgraph.0.node_weight(n2).unwrap();

        let s_joint: SphericalJointBuilder = SphericalJointBuilder::new()
            .local_anchor1((n2_weight.pos-n1_weight.pos)/2.0)
            .local_anchor2((n1_weight.pos-n2_weight.pos)/2.0);

        // println!("1: {:?} \n2: {:?}", (n2_weight.pos-n1_weight.pos)/2.0, (n1_weight.pos-n2_weight.pos)/2.0);

        if let Some(prev_edge) = p_edge {
            let pe_weight = pgraph.0.edge_weight(prev_edge).unwrap();
            commands.entity(edge_weight.entityid.unwrap()).insert(ImpulseJoint::new(pe_weight.entityid.unwrap(), s_joint));
        }
    }
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 10.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        RaycastPickCamera::default(),
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
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1000.0, ..default() })),
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

#[derive(Component)]
struct TestBut;

fn test(
    mut commands: Commands,
    asset_server: Res<AssetServer>, 
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                // size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
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
                        // size: Size::new(Val::Px(80.0), Val::Px(30.0)),
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
        }).insert(crate::Observer);
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
            Interaction::Pressed => {
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

fn pause_sim(
    input: Res<Input<KeyCode>>,
    mut config: ResMut<RapierConfiguration>,
) {
    if input.just_pressed(KeyCode::Q) {
        config.physics_pipeline_active = !config.physics_pipeline_active;
    }
}