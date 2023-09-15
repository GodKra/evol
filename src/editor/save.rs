use bevy::prelude::*;

use crate::pgraph::*;

/// System that saves the joint structure to a data file (currently ./points.ron)
/// 
/// *active
pub fn save(
    mut pgraph: ResMut<PGraph>,
) {
    for edge in pgraph.0.edge_weights_mut() {
        edge.muscle_data = edge.muscles.keys().copied().collect();
    }
    std::fs::write(
        "./pgraph.ron", 
        ron::ser::to_string_pretty(
            &pgraph.0, 
                ron::ser::PrettyConfig::new()
                .depth_limit(2)
                .separate_tuple_members(true)
                .enumerate_arrays(true)
        ).unwrap()
    ).unwrap();

    println!("** SAVED");
}