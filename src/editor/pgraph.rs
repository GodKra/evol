
use bevy::{prelude::*, utils::HashMap};
use bevy_mod_picking::{PickableMesh};
use serde::{Serialize, Deserialize};
use petgraph::{graph::*, stable_graph::StableUnGraph};

use crate::{
    editor::{selection::*, Editable}
};

use crate::util::{JointMaterial, JointMeshes, Errors};

use super::{EditMode, muscle::Muscle};

/// Graph describing the joints, connections and muscles.
pub type PointGraph = StableUnGraph<Point, Connection>;

/// Node of point graph.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Point {
    #[serde(skip)]
    pub entityid: Option<Entity>,
    pub pos: Vec3,
    pub parent: Option<NodeIndex>,
}

/// Edge of point graph.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Connection {
    #[serde(skip)]
    pub entityid: Option<Entity>,
    #[serde(skip)]
    pub muscles: HashMap<EdgeIndex, Entity>, 
    pub muscle_data: Vec<EdgeIndex>, // use hashmap with value as muscle weight later?
}


/// Component for each joint entity.
#[derive(Clone, Debug, Default, Component)]
pub struct Joint {
    pub node_index: NodeIndex,
}

/// Component for each connector entity.
#[derive(Clone, Debug, Default, Component)]
pub struct Connector {
    pub edge_index: EdgeIndex,
}

/// PointGraph struct stored as resource.
#[derive(Default, Debug, Clone, Serialize, Deserialize, Resource)]
pub struct PGraph(pub PointGraph);

impl PGraph {
    /// Spawns all the joints, connectors and muscles contained in the point graph.
    pub fn create(
        &mut self,
        commands: &mut Commands,
        meshes: Res<JointMeshes>,
        materials: Res<JointMaterial>,
    ) {
        let nodes: Vec<NodeIndex> = self.0.node_indices().collect(); // have to do this because borrowchecker
        let mut muscles_complete: HashMap<EdgeIndex, HashMap<EdgeIndex, Entity>> = HashMap::new();

        for node in nodes {
            let mut point_data = self.0.node_weight_mut(node).unwrap();

            let joint = Joint { node_index: node};

            let e = create_joint(
                commands, 
                &meshes, 
                &materials, 
                point_data.pos, 
                Some(joint),
                None,
            );

            point_data.entityid = Some(e);
        }

        let edges: Vec<EdgeIndex> = self.0.edge_indices().collect();

        for edge in edges {
            let (n1, n2) = self.0.edge_endpoints(edge).unwrap();
            let pos1 = self.0.node_weight(n1).unwrap().pos;
            let pos2 = self.0.node_weight(n2).unwrap().pos;

            let mut edge_data = self.0.edge_weight_mut(edge).unwrap();
            let connector = Connector { edge_index: edge };

            let e = create_connector(
                commands, 
                &meshes, 
                &materials, 
                pos1, 
                pos2, 
                Some(connector)
            );
            edge_data.entityid = Some(e);

            for muscle_pair in edge_data.muscle_data.iter() {
                if let Some(muscle_map) = muscles_complete.get(muscle_pair) {
                    let muscle = muscle_map.get(&edge).unwrap();
                    edge_data.muscles.insert(*muscle_pair, *muscle);
                    continue;
                }
                let muscle = commands.spawn((
                    PbrBundle {
                        mesh: meshes.connector.clone(),
                        material: materials.muscle_color.clone(),
                        ..Default::default()
                    },
                    Muscle { anchor1: Some(edge), anchor2: Some(*muscle_pair) },
                    PickableMesh::default(),
                )).id();
                commands.entity(muscle).insert(Selectable::with_type(SelectableEntity::Muscle(muscle)));
                // muscles_complete.insert(edge, muscle);
                muscles_complete.entry(edge).or_insert(HashMap::new()).insert(*muscle_pair, muscle);
                edge_data.muscles.insert(*muscle_pair, muscle);
            }
        }
    }

    /// Converts a given node index to its respective entity id if it exists.
    pub fn node_to_entity(&self, node: NodeIndex) -> Option<Entity> {
        let Some(pdata) = self.0.node_weight(node) else {
            println!("{}", Errors::NodeMissing(node));
            return None;
        };
        pdata.entityid
    }
    /// Converts a given edge index to its respective entity id if it exists.
    pub fn edge_to_entity(&self, edge: EdgeIndex) -> Option<Entity> {
        let Some(pdata) = self.0.edge_weight(edge) else {
            println!("{}", Errors::EdgeMissing(edge));
            return None;
        };
        pdata.entityid
    }
    /// Returns the node index of a given node's parent if it exists.
    pub fn node_parent(&self, node: NodeIndex) -> Option<NodeIndex> {
        let Some(pdata) = self.0.node_weight(node) else {
            println!("{}", Errors::NodeMissing(node));
            return None;
        };
        pdata.parent
    }
    /// Convenience function to get a node's parent to as entity.
    pub fn node_parent_entity(&self, node: NodeIndex) -> Option<Entity> {
        let Some(parent) = self.node_parent(node) else {
            return None;
        };
        self.node_to_entity(parent)
    }
}

/// Creates a joint with the given position, joint data and edit mode. The Joint component should
/// be manually assigned later if no joint data is passed.
pub fn create_joint(
    commands: &mut Commands,
    meshes: &Res<JointMeshes>,
    materials: &Res<JointMaterial>,
    pos: Vec3,
    joint_data: Option<Joint>,
    edit_mode: Option<EditMode>,
) -> Entity {
    let e = commands.spawn(
        (
            PbrBundle {
                mesh: meshes.head.clone(),
                material: materials.joint_color.clone(),
                transform: Transform::from_translation(pos),
                ..Default::default()
            },
            PickableMesh::default(),
            Editable{ mode: edit_mode },
            crate::Editor,
        )
    ).id();
    commands.entity(e).insert(Selectable::with_type(SelectableEntity::Joint(e)));
    if let Some(joint) = joint_data {
        commands.entity(e).insert(joint);
    }
    e
}

/// Creates a connector between the given joint positions with the connector data. The Connector component should
/// be manually assigned later if no connector data is passed.
pub fn create_connector(
    commands: &mut Commands,
    meshes: &Res<JointMeshes>,
    materials: &Res<JointMaterial>,
    joint1_pos: Vec3,
    joint2_pos: Vec3,
    connector_data: Option<Connector>,
) -> Entity {
    let r_pos = joint1_pos - joint2_pos;

    let rotation = Quat::from_rotation_arc(Vec3::Y, r_pos.normalize());
    let scale = Vec3::from([1.0, 1.0, 1.0]);
    let rotate = Mat4::from_scale_rotation_translation(scale, rotation, joint2_pos);
    
    let len = r_pos.length();
    let scale = Vec3::from([1.0, len/2.0, 1.0]);
    let translation = Vec3::new(0.0, len/2.0, 0.0);
    let position = Mat4::from_scale_rotation_translation(scale, Quat::default(), translation);

    let e = commands.spawn((
        PbrBundle {
            mesh: meshes.connector.clone(),
            material: materials.connector_color.clone(),
            transform: Transform::from_matrix(rotate * position),
            ..Default::default()
        },
        PickableMesh::default(),
    )).id();
    commands.entity(e).insert(Selectable::with_type(SelectableEntity::Connector(e)));
    if let Some(conn) = connector_data {
        commands.entity(e).insert(conn);
    }
    e
}

pub fn deserialize_pgraph(
    mut commands: Commands,
    mut graph: ResMut<PGraph>,
    meshes: Res<JointMeshes>,
    materials: Res<JointMaterial>,
) {
    let graph_data = &std::fs::read("./pgraph.ron").unwrap();
    graph.0 = ron::de::from_bytes(graph_data).unwrap();
    println!("** GENERATED: {:?}", graph.0);
    graph.create(&mut commands, meshes, materials);

}