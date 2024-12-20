use bevy::prelude::*;

use crate::structure::Structure;

#[derive(Event)]
pub struct SaveEvent;

/// System that saves the joint structure to a file.
pub fn save(
    _: Trigger<SaveEvent>,
    mut structure: ResMut<Structure>,
) {
    for edge in structure.0.edge_weights_mut() {
        edge.muscle_data = edge.muscles.keys().copied().collect();
    }
    std::fs::write(
        "./structure.ron", 
        ron::ser::to_string_pretty(
            &structure.0, 
                ron::ser::PrettyConfig::new()
                .depth_limit(2)
                .separate_tuple_members(true)
                .enumerate_arrays(true)
        ).unwrap()
    ).unwrap();

    info!(":: Saved to ./structure.ron");
}