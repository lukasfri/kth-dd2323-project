use nalgebra::{Vector2, Vector3};
use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};

use crate::{scene::Scene, tile_data::TileData};

pub struct Tile<'a> {
    pub data: Option<&'a TileData>,
    pub possible_tiles: Vec<&'a TileData>,
    pub tile_position: Vector2<usize>,
}

impl<'a> Tile<'a> {
    pub fn new(possible_tiles: Vec<&'a TileData>, position: Vector2<usize>) -> Self {
        Self {
            data: None,
            possible_tiles,
            tile_position: position,
        }
    }

    pub fn collapse(&mut self, scene: &mut Scene) {
        let weights: Vec<u32> = self.possible_tiles.iter().map(|t| t.weight).collect();
        let mut rng = thread_rng();
        let choosen_tile = WeightedIndex::new(weights).unwrap().sample(&mut rng);

        // Place tile
        self.data = Some(self.possible_tiles[choosen_tile]);
        self.possible_tiles = vec![];
        scene.instantiate_model(
            &self.data.unwrap().model,
            Vector3::<f32>::new(
                self.tile_position.x as f32,
                0.0,
                self.tile_position.y as f32,
            ),
        );
    }
}
