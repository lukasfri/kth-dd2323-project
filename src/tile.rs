use nalgebra::{Vector2, Vector3};
use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};

use crate::{scene::Scene, tile_data::TileData, Direction};

pub struct Tile<'a> {
    pub data: Option<&'a TileData>,
    pub possible_tiles: Vec<&'a TileData>,
    pub tile_position: Vector2<usize>, // Position in grid
}

impl<'a> Tile<'a> {
    pub fn new(possible_tiles: Vec<&'a TileData>, position: Vector2<usize>) -> Self {
        Self {
            data: None,
            possible_tiles,
            tile_position: position,
        }
    }

    pub fn collapse(&mut self, scene: &mut Scene) -> bool {
        // Already collapsed
        if self.data.is_some() {
            return false;
        }

        let weights: Vec<u32> = self.possible_tiles.iter().map(|t| t.weight).collect();
        // Impossible to collapse
        if weights.is_empty() {
            return false;
        }

        let mut rng = thread_rng();
        let choosen_tile = WeightedIndex::new(weights).unwrap().sample(&mut rng);

        // Place tile
        self.data = Some(self.possible_tiles[choosen_tile]);
        self.possible_tiles = vec![];
        scene.instantiate_model(
            &self.data.unwrap().model,
            Vector3::<f32>::new(
                self.tile_position.x as f32,
                self.tile_position.y as f32,
                0.0,
            ),
        );

        true
    }

    // Remove options after neighbouring tile has changed
    // Direction is the the direction of this tiles edge that should be checked
    // Edge is the new edge type of that edge
    // Returns: impossible to collapse
    pub fn remove_options(&mut self, direction: Direction, edge: &str) -> bool {
        self.possible_tiles = self
            .possible_tiles
            .iter()
            .copied()
            .filter(|tile| tile.check_edge(direction, edge))
            .collect::<Vec<&'a TileData>>();
        self.possible_tiles.is_empty()
    }
}
