use std::ops::{Deref, DerefMut};

use bevy::{prelude::*, utils::HashMap};

use serde::{Serialize, Deserialize};
use petgraph::{graph::*, stable_graph::StableUnGraph};

use crate::{selection::{Selectable, SelectableEntity}, util::{Errors, JointMaterial, JointMeshes}};

/// Node of body graph. These are joints. `entityid` is the bevy-given EntityID, `pos` is the 3D position vector, and parent is the parent node.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Point {
    #[serde(skip)]
    pub entityid: Option<Entity>,
    pub pos: Vec3,
    pub parent: Option<NodeIndex>,
}

/// Edge of structure graph. These are connectors. `entityid` is the bevy-given EntityID, `muscles` maps UNKNOWN
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

/// Component to each muscle describing its anchors (edges/connectors).
#[derive(Component, Default, Debug)]
pub struct Muscle {
    pub anchor1: Option<EdgeIndex>,
    pub anchor2: Option<EdgeIndex>,
}

/// Graph resource describing the joints (nodes), connections (edges) and muscles (in the edge weights).
#[derive(Default, Debug, Clone, Serialize, Deserialize, Resource)]
pub struct Structure(pub StableUnGraph<Point, Connection>);

impl Deref for Structure {
    type Target = StableUnGraph<Point, Connection>;

    fn deref(&self) -> &Self::Target {
        &self.0    
    }
}

impl DerefMut for Structure {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Structure {
    /// Spawns all the joints, connectors and muscles contained in the structure graph where `#_components` are extra
    /// components that may be desired to be added to the entities when deserializing the graph, and `state` is the game 
    /// state where the object exists.
    pub fn create<J, M, C, S>(
        &mut self,
        commands: &mut Commands,
        meshes: Res<JointMeshes>,
        materials: Res<JointMaterial>,
        joint_components: J,
        conn_components: C,
        muscle_components: M,
        state: S,
    ) where 
        J: Bundle + Clone, // joint components
        M: Bundle + Clone, // muscle components
        C: Bundle + Clone, // connector components
        S: Bundle + Copy // game state
    {
        let nodes: Vec<NodeIndex> = self.node_indices().collect(); // have to do this because borrowchecker
        let mut muscles_complete: HashMap<EdgeIndex, HashMap<EdgeIndex, Entity>> = HashMap::new();

        for node in nodes {
            let point_data = self.node_weight_mut(node).unwrap();

            let joint = Joint { node_index: node};

            let e = create_joint(
                commands, 
                &meshes, 
                &materials, 
                point_data.pos, 
                Some(joint),
                joint_components.clone(),
                state
            );

            point_data.entityid = Some(e);
        }

        let edges: Vec<EdgeIndex> = self.edge_indices().collect();

        for edge in edges {
            let (n1, n2) = self.edge_endpoints(edge).unwrap();
            let pos1 = self.node_weight(n1).unwrap().pos;
            let pos2 = self.node_weight(n2).unwrap().pos;

            let edge_data = self.edge_weight_mut(edge).unwrap();
            let connector = Connector { edge_index: edge };

            let e = create_connector(
                commands, 
                &meshes, 
                &materials, 
                pos1, 
                pos2, 
                Some(connector),
                conn_components.clone(),
                state
            );
            edge_data.entityid = Some(e);

            for muscle_pair in edge_data.muscle_data.iter() {
                if let Some(muscle_map) = muscles_complete.get(muscle_pair) {
                    let muscle = muscle_map.get(&edge).unwrap();
                    edge_data.muscles.insert(*muscle_pair, *muscle);
                    continue;
                }
                let muscle = create_muscle(
                    commands, 
                    &meshes, 
                    &materials,
                    Some(edge),
                    Some(*muscle_pair),
                    muscle_components.clone(),
                    state
                );
                muscles_complete.entry(edge).or_insert(HashMap::new()).insert(*muscle_pair, muscle);
                edge_data.muscles.insert(*muscle_pair, muscle);
            }
        }
    }

    /// Converts a given node index to its respective entity id if it exists.
    pub fn node_to_entity(&self, node: NodeIndex) -> Option<Entity> {
        let Some(pdata) = self.node_weight(node) else {
            warn!("{}", Errors::NodeMissing(node));
            return None;
        };
        pdata.entityid
    }
    /// Converts a given edge index to its respective entity id if it exists.
    pub fn edge_to_entity(&self, edge: EdgeIndex) -> Option<Entity> {
        let Some(pdata) = self.edge_weight(edge) else {
            warn!("{}", Errors::EdgeMissing(edge));
            return None;
        };
        pdata.entityid
    }
    /// Returns the node index of a given node's parent if it exists.
    pub fn node_parent(&self, node: NodeIndex) -> Option<NodeIndex> {
        let Some(pdata) = self.node_weight(node) else {
            warn!("{}", Errors::NodeMissing(node));
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
/// be manually assigned later if no joint data is passed. `components` are extra components to be added if desired,
/// and `state` is the game state where the object exists.
pub fn create_joint<J: Bundle, S: Bundle>(
    commands: &mut Commands,
    meshes: &Res<JointMeshes>,
    materials: &Res<JointMaterial>,
    pos: Vec3,
    joint_data: Option<Joint>,
    components: J,
    state: S,
) -> Entity {
    let e = commands.spawn(
        (
            Mesh3d(meshes.head.clone()),
            MeshMaterial3d(materials.joint_color.clone()),
            Transform::from_translation(pos),
            components,
            state,
        )
    ).id();
    commands.entity(e).insert(Selectable::with_type(SelectableEntity::Joint(e)));
    if let Some(joint) = joint_data {
        commands.entity(e).insert(joint);
    }
    info!(":: Created Joint: {:?}", e);
    e
}

/// Creates a connector between the given joint positions with the connector data. The Connector component should
/// be manually assigned later if no connector data is passed. `state` is the game state where the object exists.
pub fn create_connector<C: Bundle, S: Bundle>(
    commands: &mut Commands,
    meshes: &Res<JointMeshes>,
    materials: &Res<JointMaterial>,
    joint1_pos: Vec3,
    joint2_pos: Vec3,
    connector_data: Option<Connector>,
    components: C,
    state: S,
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
        Mesh3d(meshes.connector.clone()),
        MeshMaterial3d(materials.connector_color.clone()),
        Transform::from_matrix(rotate * position),
        components,
        state,
        
        // RaycastPickTarget::default(),
    )).id();
    commands.entity(e).insert(Selectable::with_type(SelectableEntity::Connector(e)));
    if let Some(conn) = connector_data {
        commands.entity(e).insert(conn);
    }
    info!(":: Created Connector: {:?}", e);
    e
}

/// Creates a muscle with the given anchors. `components` are extra components to be added if needed, 
/// `state` is the game state where the object exists.
pub fn create_muscle<M: Bundle, S: Bundle> (
    commands: &mut Commands,
    meshes: &Res<JointMeshes>,
    materials: &Res<JointMaterial>,
    anchor1: Option<EdgeIndex>,
    anchor2: Option<EdgeIndex>,
    components: M,
    state: S,
) -> Entity{
    let e = commands.spawn((
        Mesh3d(meshes.connector.clone()),
        MeshMaterial3d(materials.muscle_color.clone()),
        Muscle { anchor1, anchor2 },
        components,
        state,
        
    )).id();
    commands.entity(e).insert(Selectable::with_type(SelectableEntity::Muscle(e)));
    info!(":: Created Muscle: {:?}", e);
    e
}