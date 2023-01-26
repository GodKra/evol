// use bevy::{prelude::*};
// use serde::{Serialize, Deserialize};

// use bevy_rapier3d::prelude::*;

// use crate::util::{JointMaterial, JointMeshes};


// #[derive(Clone, Debug, Default, Serialize, Deserialize)]
// pub struct Point {
//     pub r_coords: (f32, f32, f32),
//     pub connections: Vec<Point>,
// }

// /// Physical manifestation of the Point struct.
// #[derive(Clone, Debug, Default, Component)]
// pub struct Joint {
//     pub parent: Option<Entity>,
//     pub rotator: Option<Entity>,
//     pub connector: Option<Entity>,   
// }

// #[derive(Component)]
// pub struct Root;

// impl Point {
//     pub fn create_object(
//         &mut self,
//         commands: &mut Commands,
//         meshes: &Res<JointMeshes>,
//         materials: &Res<JointMaterial>,
//         parent: Option<Entity>,
//         global_pos: Vec3,
//     ) {
//         let pos = Vec3::new(self.r_coords.0, self.r_coords.1, self.r_coords.2);
//         let joint = create_joint(
//             parent, 
//             global_pos,
//             pos, 
//             commands, 
//             meshes, 
//             materials
//         );
//         for connection in &mut self.connections {
//             connection.create_object(commands, meshes, materials, Some(joint), global_pos+pos);
//         }
//     }
// }

// pub fn generate_mesh(
//     mut commands: Commands,
//     meshes: Res<JointMeshes>,
//     materials: Res<JointMaterial>,
// ) {
//     let point_data = &std::fs::read("./points.ron").unwrap();
//     // let point_data = include_bytes!("../assets/points.ron");
//     let mut points: Point = ron::de::from_bytes(point_data).unwrap_or_default();
//     println!("{:?}", points);
    
//     points.create_object(&mut commands, &meshes, &materials, None, Vec3::ZERO);
// }

// /// Creates a joint with the given relative position from the given global position. A spherical joint is created
// /// between the joint and its parent.
// pub fn create_joint(
//     mut parent: Option<Entity>,
//     global_pos: Vec3,
//     position: Vec3,
//     commands: &mut Commands,
//     meshes: &Res<JointMeshes>,
//     materials: &Res<JointMaterial>,
// ) -> Entity {
//     let mut joint = Joint::default();

//     if parent.is_none() {
//         parent = Some(commands.spawn((
//             PbrBundle {
//                 mesh: meshes.head.clone(),
//                 material: materials.joint_color.clone(),
//                 transform: Transform::from_translation(global_pos+position),
//                 ..Default::default()
//             },
//             Root,
//             RigidBody::Dynamic,
//             Collider::ball(1.0),
//             Friction::coefficient(3.0),
//             crate::Observer,
//             joint,
//         )).id())
//     } else {
//         let len = position.length();
//         let scale = Vec3::from([1.0, 1.0, 1.0]);
//         let rotation = Quat::from_rotation_arc(Vec3::Y, position.normalize());
        
//         let rotator = Some(commands.spawn((
//             PbrBundle {
//                 transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, Vec3::default())),
//                 ..Default::default()
//             },
//             crate::Observer,
//         )).with_children(|p| {
//             // connector
//             let scale = Vec3::from([1.0, len/2.0, 1.0]);
//             let rotation = Quat::default();
//             let translation = Vec3::new(0.0, len/2.0, 0.0);

//             joint.connector = Some(p.spawn(PbrBundle {
//                 mesh: meshes.connector.clone(),
//                 material: materials.connector_color.clone(),
//                 transform: Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, translation)),
//                 ..Default::default()
//             }).id());
//         }).id());
//         joint.parent = parent;
//         joint.rotator = rotator;

//         let s_joint = SphericalJointBuilder::new()
//             .local_anchor1(Vec3::ZERO)
//             .local_anchor2(-position);

//         let current = commands.spawn(PbrBundle {
//             mesh: meshes.head.clone(),
//             material: materials.joint_color.clone(),
//             transform: Transform::from_translation(global_pos+position),
//             ..Default::default()
//         })
//         //.insert(BoundVol::default())
//         .insert(joint)
//         .insert(RigidBody::Dynamic)
//         .insert(Collider::ball(1.0))
//         .insert(ImpulseJoint::new(parent.unwrap(), s_joint))
//         .insert(Friction::coefficient(3.0))
//         // .insert(Restitution::coefficient(1.0))
//         .insert(crate::Observer)
//         // .push_children(&[rotator.unwrap()])
//         .id();

//         // commands.entity(parent.unwrap()).push_children(&[rotator.unwrap()]);
//         parent = Some(current);
//     }
//     parent.unwrap()
// }

// /// Automatically updates the visual connector between the joints.
// pub fn update_connector(
//     changed_joints: Query<(&Joint, Entity)>,
//     mut transform_q: Query<&mut Transform>,
// ) {
//     for (joint, entity) in changed_joints.iter() {
//         let transform = *transform_q.get(entity).unwrap();
//         // println!("REEE");
//         if joint.rotator.is_none() {
//             continue;
//         }

//         let parent_t = *transform_q.get(joint.parent.unwrap()).unwrap();
        
//         let mut rotator = transform_q.get_mut(joint.rotator.unwrap()).unwrap();
//         rotator.translation = parent_t.translation;
        
//         let j_to_p = transform.translation-parent_t.translation;

//         rotator.rotation = Quat::from_rotation_arc(Vec3::Y, j_to_p.normalize());
        
//         let mut connector = transform_q.get_mut(joint.connector.unwrap()).unwrap();

//         let scale = Vec3::from([1.0, j_to_p.length()/2.0, 1.0]);
//         let rotation = Quat::default();
//         let translation = Vec3::new(0.0, j_to_p.length()/2.0, 0.0);
//         *connector = Transform::from_matrix(Mat4::from_scale_rotation_translation(scale, rotation, translation));
//     }
// }