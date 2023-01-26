// use bevy::prelude::*;
// use serde::{Serialize, Deserialize};

// use crate::util::{JointMaterial, JointMeshes};

// use super::muscle::{MuscleData, MuscleHalfs};
// use super::joint::{Point, IDCounter, IDMap};

// #[derive(Default, Debug, Clone, Serialize, Deserialize)]
// pub struct Body {
//     pub points: Point,
//     pub muscle: MuscleData,
// }

// impl Body {
//     pub fn create_body(
//         &mut self,
//         commands: &mut Commands,
//         meshes: Res<JointMeshes>,
//         materials: Res<JointMaterial>,
//         id_counter: &mut ResMut<IDCounter>,
//         id_map: &mut ResMut<IDMap>,
//     ) {
//         // let mut index = 0;
//         let mut muscle_halfs = MuscleHalfs::default();
//         self.points.create_points(
//             commands, &meshes, &materials,
//             id_counter,
//             id_map,
//             &self.muscle,
//             &mut muscle_halfs,
//             None,
//         );
//     }
// }

// pub fn generate_structure(
//     mut commands: Commands,
//     meshes: Res<JointMeshes>,
//     materials: Res<JointMaterial>,
//     mut id_counter: ResMut<IDCounter>,
//     mut id_map: ResMut<IDMap>,
// ) {
//     let body_data = &std::fs::read("./points.ron").unwrap();
//     let mut body: Body = ron::de::from_bytes(body_data).unwrap();
//     body.create_body(&mut commands, meshes, materials, &mut id_counter, &mut id_map);

//     println!("** GENERATED: {:?}\nID Map: {:?}", body.points, id_map);
// }